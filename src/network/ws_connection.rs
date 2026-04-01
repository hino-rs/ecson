//! 個別のWebSocket接続を処理し、ECSとのメッセージ送受信を仲介するモジュールです。

use crate::network::channels::*;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// 個別のWebSocket接続を処理し、ECS側との双方向通信を管理します。
///
/// クライアントごとに独立したタスクとして実行され、WebSocketのハンドシェイク確立後、
/// 読み取り（Read）タスクと書き込み（Write）タスクに分割して非同期処理を行います。
///
/// # 引数
/// * `stream` - 受け入れたTCPストリーム
/// * `conn_id` - この接続に割り当てられた一意のID
/// * `ecs_tx` - ECS側へネットワークイベント（接続、切断、メッセージ受信）を送るためのチャンネル
pub async fn handle_connection(
    stream: TcpStream,
    conn_id: u64,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) {
    // WebSocketハンドシェイクの実行
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during websocket handshake for ID {conn_id}: {e}");
            return;
        }
    };

    info!("WebSocket connection established for ID {conn_id}");

    // ストリームを送信（Sink）と受信（Stream）に分割
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // ECS側からこの接続に対するメッセージを受け取るための専用チャンネルを作成
    let (client_tx, mut client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);

    // Read/Writeタスク間で「切断」を伝えるためのシグナル
    let cancel = CancellationToken::new();

    // ECSへ「接続完了」イベントを通知
    // ECS側から返信するための送信チャンネル（client_tx）も一緒に渡す
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

    // 書き込み（Write）タスクの生成: ECSから送られてきたメッセージ（client_rx）を受け取り、WebSocketクライアントへ送信する
    // CancellationToken がキャンセルされたら select! で即座に抜ける
    let cancel_for_write = cancel.clone();
    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Readタスクが切断を検知したらここで抜ける
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

    // 読み取り（Read）タスク: WebSocketクライアントからのメッセージを受け取り、ECS（ecs_tx）へ転送する
    // 切断を検知したら CancellationToken をキャンセルして Write タスクを止める
    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if msg.is_close() {
                break; // 切断メッセージを受け取ったらループを抜ける
            }

            // WebSocketのメッセージを内部用のペイロードに変換
            let payload = match msg {
                Message::Text(t) => NetworkPayload::Text(t.to_string()),
                Message::Binary(b) => NetworkPayload::Binary(b.into()),
                _ => continue, // Ping/Pongなどはスキップ
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

        // Writeタスクをキャンセル (client_rxの待機を解除)
        cancel.cancel();
        // ECSへ切断を通知
        let _ = ecs_tx_clone
            .send(NetworkEvent::Disconnected { id: conn_id })
            .await;
    });

    let _ = tokio::join!(read_task, write_task);
    info!("Connection closed for ID {conn_id}");
}
