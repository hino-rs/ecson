//! Module for managing a raw UDP server.
//!
//! UDP is connectionless, so this module simulates virtual connections by tracking
//! each unique `SocketAddr` that sends a datagram. On first contact from a new peer,
//! a `NetworkEvent::Connected` is fired and a dedicated write task is spawned.
//!
//! Framing: UDP datagrams are self-delimiting — no VarInt length prefix is used.
//! Each datagram received is forwarded to ECS as a single `NetworkPayload::Binary`.
//!
//! Disconnection: UDP has no built-in disconnect signal. `NetworkEvent::Disconnected`
//! is **not** sent automatically. Applications requiring timeout-based cleanup should
//! implement their own heartbeat/eviction logic in ECS systems.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Maximum UDP payload size (IPv4 theoretical max minus IP+UDP headers).
const MAX_UDP_PACKET_SIZE: usize = 65507;

// =========================================================
// Entry Point
// =========================================================

pub async fn run(
    addr: SocketAddr,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    info!("Raw UDP server listening on {}", addr);

    // Maps peer address → (conn_id, sender to write task)
    let mut peers: HashMap<SocketAddr, (u64, mpsc::Sender<NetworkPayload>)> = HashMap::new();
    let mut buf = vec![0u8; MAX_UDP_PACKET_SIZE];

    loop {
        let (len, peer_addr) = match socket.recv_from(&mut buf).await {
            Ok(r) => r,
            Err(e) => {
                warn!("UDP recv_from error: {e}");
                continue;
            }
        };

        let data = buf[..len].to_vec();

        // ── First datagram from this peer: open a virtual connection ──
        if let std::collections::hash_map::Entry::Vacant(e) = peers.entry(peer_addr) {
            let conn_id = rand::random::<u64>();
            let (client_tx, client_rx) = mpsc::channel::<NetworkPayload>(client_buffer);

            // Spawn a dedicated write task for this peer
            tokio::spawn(write_loop(socket.clone(), client_rx, peer_addr, conn_id));

            // Notify ECS that a new client has "connected"
            if ecs_tx
                .send(NetworkEvent::Connected {
                    id: conn_id,
                    sender: client_tx.clone(),
                })
                .await
                .is_err()
            {
                warn!("ECS channel closed before Connected could be sent (ID: {conn_id})");
                return Ok(());
            }

            info!("New UDP virtual connection from {peer_addr} (ID: {conn_id})");
            e.insert((conn_id, client_tx));
        }

        // ── Forward datagram to ECS ──
        let (conn_id, _) = peers.get(&peer_addr).unwrap();
        if ecs_tx
            .send(NetworkEvent::Message {
                id: *conn_id,
                payload: NetworkPayload::Binary(data),
            })
            .await
            .is_err()
        {
            // ECS receiver dropped — shut down the server task
            break;
        }
    }

    Ok(())
}

// =========================================================
// Send Loop (one task per virtual connection)
// =========================================================

async fn write_loop(
    socket: Arc<UdpSocket>,
    mut client_rx: mpsc::Receiver<NetworkPayload>,
    peer_addr: SocketAddr,
    conn_id: u64,
) {
    while let Some(payload) = client_rx.recv().await {
        let data = match payload {
            NetworkPayload::Binary(b) => b,
            // Text is encoded as UTF-8 bytes; UDP servers should prefer Binary.
            NetworkPayload::Text(t) => t.into_bytes(),
        };

        if let Err(e) = socket.send_to(&data, peer_addr).await {
            error!("ID {conn_id}: failed to send UDP datagram to {peer_addr}: {e}");
            break;
        }
    }

    info!("UDP write loop ended for ID {conn_id} ({peer_addr})");
}
