//! WebSocketサーバーの起動と、クライアント接続の受け入れを管理するモジュールです。

use crate::ws_connection;
use ecson_ecs::channels::NetworkEvent;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::info;

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

pub async fn run(
    addr: &str,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket server listening on ws://{addr}");

    while let Ok((stream, addr)) = listener.accept().await {
        let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        info!("New connection from: {addr} (ID: {conn_id})");
        tokio::spawn(ws_connection::handle_connection(
            stream,
            conn_id,
            ecs_tx.clone(),
            client_buffer,
        ));
    }

    Ok(())
}
