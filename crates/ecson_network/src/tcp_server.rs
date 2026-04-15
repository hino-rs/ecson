//! Module for managing the startup of a raw TCP server and accepting client connections.
//! Does not perform WebSocket handshakes, directly handling raw TCP streams.
//! Provides the foundation for servers using custom binary protocols like Minecraft.

use std::net::SocketAddr;

use crate::tcp_connection;
use ecson_ecs::channels::NetworkEvent;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub async fn run(
    addr: SocketAddr,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("Raw TCP server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let conn_id = rand::random::<u64>();
                info!("New TCP connection from: {peer_addr} (ID: {conn_id})");

                tokio::spawn(tcp_connection::handle_connection(
                    stream,
                    conn_id,
                    ecs_tx.clone(),
                    client_buffer,
                ));
            }
            Err(e) => {
                // A failure in accept does not stop the entire server; it only logs the error and continues processing.
                warn!("Failed to accept TCP connection: {e}");
            }
        }
    }
}
