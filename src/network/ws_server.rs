//! WebSocketサーバーの起動と、クライアント接続の受け入れを管理するモジュールです。

use tracing::info;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use crate::network::{ws_connection, channels::NetworkEvent};
use std::sync::atomic::{AtomicU64, Ordering};

/// クライアント接続ごとに一意のIDを生成するための、スレッドセーフなカウンター。
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// WebSocketサーバーを起動し、クライアントからの接続を非同期で待機します。
/// 
/// 接続が確立されるたびに一意のIDを割り振り、新しいTokioタスクを生成して
/// `handle_connection` へ処理を委譲します。
/// 
/// # 引数
/// * `addr` - サーバーがリッスンするアドレス（例: "127.0.0.1:8080"）
/// * `ecs_tx` - Tokio側のネットワークイベントをECS側へ伝達するための送信チャンネル
pub async fn run(
    addr: &str, 
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket server listening on ws://{addr}");

    // 接続の受け入れループ
    while let Ok((stream, addr)) = listener.accept().await {
        // 新しい接続が来るたびにIDを1進めて取得
        let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);

        info!("New connection from: {addr} (ID: {conn_id})");

        // ECS送信用のチャンネルをクローンし、各コネクション処理タスクへ渡す
        tokio::spawn(ws_connection::handle_connection(stream, conn_id, ecs_tx.clone(), client_buffer,));
    }
    
    Ok(())
}