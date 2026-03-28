use crate::network::channels::*;
use tokio::sync::mpsc;
use wtransport::Connection;

pub async fn handle_connection(
    connection: Connection,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
) {
    // ECSから受け取る用のチャンネル
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(100);

    // 接続イベントをECSへ
    if ecs_tx.send(NetworkEvent::Connected {
        id: conn_id, 
        sender: client_tx,
    }).await.is_err() {
        return;
    }

    // wtransportのConnectionを送受信用に分ける
    let conn_for_send = connection.clone();
    let conn_for_recv = connection.clone();
    let ecs_tx_clone = ecs_tx.clone();

    // 送信タスク
    let send_task = tokio::spawn(async move {
        while let Some(payload) = client_rx.recv().await {
            match payload {
                // データグラムとして送信
                NetworkPayload::Text(t) => {
                    let _ = conn_for_send.send_datagram(t.into_bytes());
                }
                NetworkPayload::Binary(b) => {
                    let _ = conn_for_send.send_datagram(b);
                }
            }            
        }
    });

    // 受信タスク
    let recv_task = tokio::spawn(async move {
        // データグラムの受信待ち
        while let Ok(datagram) = conn_for_recv.receive_datagram().await {
            let bytes = datagram.to_vec();

            // ペイロードに変換してECSへ
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
        let _ = ecs_tx_clone.send(NetworkEvent::Disconnected { id: conn_id }).await;
    });

    let _ = tokio::join!(send_task, recv_task);
    println!("WebTransport Connection closed for ID {conn_id}");
}