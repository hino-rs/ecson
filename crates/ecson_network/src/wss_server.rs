//! Module for starting the TLS-secured WebSocket server (WSS) and accepting incoming connections.

use std::net::SocketAddr;

use crate::ws_connection;
use ecson_ecs::channels::NetworkEvent;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};

pub async fn run(
    addr: SocketAddr,
    acceptor: TlsAcceptor,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket(TLS) server listening on wss://{}", addr);

    while let Ok((tcp_stream, peer_addr)) = listener.accept().await {
        let conn_id = rand::random::<u64>();
        let acceptor = acceptor.clone();
        let ecs_tx = ecs_tx.clone();

        info!("New TCP connection from: {peer_addr} (ID: {conn_id}), starting TLS handshake...");

        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(tcp_stream).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("TLS handshake failed for ID {conn_id} ({peer_addr}): {e}");
                    return;
                }
            };

            info!("TLS handshake success for ID {conn_id}");
            ws_connection::handle_connection(tls_stream, conn_id, ecs_tx, client_buffer).await;
        });
    }

    Ok(())
}
