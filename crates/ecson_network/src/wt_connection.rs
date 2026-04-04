//! WebTransportクライアントとの個別の接続を処理し、ECSとのメッセージ送受信を仲介するモジュールです。

use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use wtransport::Connection;
use tracing::info;

pub async fn handle_connection(
    connection: Connection,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) {
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);
    let cancel = CancellationToken::new();

    if ecs_tx.send(NetworkEvent::Connected {
        id: conn_id,
        sender: client_tx,
    }).await.is_err() {
        return;
    }

    let conn_for_send = connection.clone();
    let conn_for_recv = connection.clone();
    let ecs_tx_clone = ecs_tx.clone();

    let cancel_for_send = cancel.clone();
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_for_send.cancelled() => break,
                payload = client_rx.recv() => {
                    match payload {
                        Some(p) => {
                            match p {
                                NetworkPayload::Text(t) => {
                                    let _ = conn_for_send.send_datagram(t.into_bytes());
                                }
                                NetworkPayload::Binary(b) => {
                                    let _ = conn_for_send.send_datagram(b);
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Ok(datagram) = conn_for_recv.receive_datagram().await {
            let bytes = datagram.to_vec();

            let payload = match String::from_utf8(bytes) {
                Ok(text) => NetworkPayload::Text(text),
                Err(e) => NetworkPayload::Binary(e.into_bytes()),
            };

            if ecs_tx_clone
                .send(NetworkEvent::Message { id: conn_id, payload })
                .await
                .is_err()
            {
                break;
            }
        }

        cancel.cancel();
        let _ = ecs_tx_clone.send(NetworkEvent::Disconnected { id: conn_id }).await;
    });

    let _ = tokio::join!(send_task, recv_task);
    info!("WebTransport Connection closed for ID {conn_id}");
}
