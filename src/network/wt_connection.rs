//! WebTransportクライアントとの個別の接続を処理し、ECSとのメッセージ送受信を仲介するモジュールです。

use crate::network::channels::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use wtransport::Connection;

/// 個別のWebTransport接続を処理し、ECS側との双方向通信を管理します。
///
/// WebSocket版と同様にクライアントごとに独立したタスクとして実行されますが、
/// こちらはWebTransportの「データグラム（Datagram）」を使用した通信を行っています。
/// 
/// # 引数
/// * `connection` - 確立されたWebTransportのコネクション
/// * `conn_id` - この接続に割り当てられた一意のID
/// * `ecs_tx` - ECS側へネットワークイベントを送るためのチャンネル
pub async fn handle_connection(
    connection: Connection,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) {
    // ECS側からこの接続に対するメッセージを受け取るための専用チャンネルを作成
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);

    // Read/Send タスク間で「切断」を伝えるためのシグナル
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

    // Send タスク: ECS からのペイロードをデータグラムとして送信する
    // CancellationToken がキャンセルされたら select! で即座に抜ける
    let cancel_for_send = cancel.clone();
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Recv タスクが切断を検知したらここで抜ける
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
                        // client_tx が drop された（= ECS 側がエンティティを破棄した）
                        None => break,
                    }
                }
            }
        }
    });

    // Recv タスク: クライアントからのデータグラムを受信して ECS へ転送する
    // 切断を検知したら CancellationToken をキャンセルして Send タスクを止める
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

        // ① Send タスクをキャンセル（client_rx の待機を解除）
        cancel.cancel();
        // ② ECS へ切断を通知
        let _ = ecs_tx_clone.send(NetworkEvent::Disconnected { id: conn_id }).await;
    });

    let _ = tokio::join!(send_task, recv_task);
    println!("WebTransport Connection closed for ID {conn_id}");
}