//! 個別のWebSocket接続を処理し、ECSとのメッセージ送受信を仲介するモジュールです。

use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub async fn handle_connection<S>(
    stream: S,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) where
    S: AsyncRead + AsyncWrite + Unpin + 'static + std::marker::Send,
{
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during websocket handshake for ID {conn_id}: {e}");
            return;
        }
    };

    info!("WebSocket connection established for ID {conn_id}");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);
    let cancel = CancellationToken::new();

    if ecs_tx
        .send(NetworkEvent::Connected {
            id: conn_id,
            sender: client_tx,
        })
        .await
        .is_err()
    {
        return;
    }

    let cancel_for_write = cancel.clone();
    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_for_write.cancelled() => break,
                payload = client_rx.recv() => {
                    match payload {
                        Some(p) => {
                            let ws_msg = match p {
                                NetworkPayload::Text(t) => Message::Text(t.into()),
                                NetworkPayload::Binary(b) => Message::Binary(b.into()),
                            };
                            if ws_sender.send(ws_msg).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let ecs_tx_clone = ecs_tx.clone();
    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if msg.is_close() {
                break;
            }

            let payload = match msg {
                Message::Text(t) => NetworkPayload::Text(t.to_string()),
                Message::Binary(b) => NetworkPayload::Binary(b.into()),
                _ => continue,
            };

            if ecs_tx_clone
                .send(NetworkEvent::Message {
                    id: conn_id,
                    payload,
                })
                .await
                .is_err()
            {
                break;
            }
        }

        cancel.cancel();
        let _ = ecs_tx_clone
            .send(NetworkEvent::Disconnected { id: conn_id })
            .await;
    });

    let _ = tokio::join!(read_task, write_task);
    info!("Connection closed for ID {conn_id}");
}
