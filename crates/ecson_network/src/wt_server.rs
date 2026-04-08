//! WebTransportサーバーの起動と、クライアント接続の受け入れを管理するモジュールです。

use crate::wt_connection;
use ecson_ecs::channels::NetworkEvent;
use tokio::sync::mpsc;
use tracing::{error, info};
use wtransport::{Endpoint, Identity, ServerConfig};

pub async fn run(
    addr: &str,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"])?;

    println!(
        "Certificate Hash: {:?}",
        identity.certificate_chain().as_slice()[0]
            .hash()
            .fmt(wtransport::tls::Sha256DigestFmt::BytesArray)
    );

    let config = ServerConfig::builder()
        .with_bind_address(addr.parse()?)
        .with_identity(identity)
        .build();

    let endpoint = Endpoint::server(config)?;
    info!("WebTransport server listening on https://{addr}");

    loop {
        let incoming_session = endpoint.accept().await;
        let conn_id = rand::random::<u64>();
        let ecs_tx_clone = ecs_tx.clone();

        tokio::spawn(async move {
            match incoming_session.await {
                Ok(session_request) => match session_request.accept().await {
                    Ok(connection) => {
                        info!("New WebTransport connection established (ID: {conn_id})");
                        wt_connection::handle_connection(
                            connection,
                            conn_id,
                            ecs_tx_clone,
                            client_buffer,
                        )
                        .await;
                    }
                    Err(e) => error!("WebTransport session accept error for ID {conn_id}: {e}"),
                },
                Err(e) => error!("Incoming WebTransport connection error: {e}"),
            }
        });
    }
}
