//! WebTransportクライアントとの個別の接続を処理し、ECSとのメッセージ送受信を仲介するモジュールです。

use crate::network::channels::*;
use tokio::sync::mpsc;
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
) {
    // ECS側からこの接続に対するメッセージを受け取るための専用チャンネルを作成
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(100);

    // 1. ECSへ「接続完了」イベントを通知
    // 合わせてECSから返信するための送信チャンネル（client_tx）を渡す
    if ecs_tx.send(NetworkEvent::Connected {
        id: conn_id, 
        sender: client_tx,
    }).await.is_err() {
        return;
    }

    // WebTransportのConnectionはClone可能で、スレッドセーフに送受信を分割できます
    let conn_for_send = connection.clone();
    let conn_for_recv = connection.clone();
    let ecs_tx_clone = ecs_tx.clone();

    // 2. 送信（Write）タスクの生成
    // ECSから送られてきたペイロードを、WebTransportのデータグラムとして送信します
    let send_task = tokio::spawn(async move {
        while let Some(payload) = client_rx.recv().await {
            // ※データグラムでの送信のため、TCPのような到達保証や順序保証はありません
            match payload {
                NetworkPayload::Text(t) => {
                    let _ = conn_for_send.send_datagram(t.into_bytes());
                }
                NetworkPayload::Binary(b) => {
                    let _ = conn_for_send.send_datagram(b);
                }
            }            
        }
    });

    // 3. 受信（Read）タスクの生成
    // クライアントから送られてきたデータグラムを受信し、ECSへ転送します
    let recv_task = tokio::spawn(async move {
        // データグラムの受信待ちループ
        while let Ok(datagram) = conn_for_recv.receive_datagram().await {
            let bytes = datagram.to_vec();

            // 試行的にUTF-8のテキストとしてパースし、失敗した場合はバイナリとして扱う
            let payload = match String::from_utf8(bytes) {
                Ok(text) => NetworkPayload::Text(text),
                Err(e) => NetworkPayload::Binary(e.into_bytes()), // エラーから元のバイト列を回収
            };

            // ECSへメッセージ受信イベントを送信
            if ecs_tx_clone
                .send(NetworkEvent::Message { id: conn_id, payload })
                .await
                .is_err()
            {
                break; // ECS側がシャットダウン等の理由で受け取れなければループを抜ける
            }
        }
        
        // ループを抜けた（＝切断された）場合、ECSへ「切断」イベントを通知
        let _ = ecs_tx_clone.send(NetworkEvent::Disconnected { id: conn_id }).await;
    });

    // ReadタスクとWriteタスクの両方が完了するまで待機
    let _ = tokio::join!(send_task, recv_task);
    println!("WebTransport Connection closed for ID {conn_id}");
}