use tokio::sync::mpsc;
use wtransport::Identity;
use wtransport::{Endpoint, ServerConfig};
use crate::network::channels::NetworkEvent;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::network::wt_connection;

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

pub async fn run(addr: &str, ecs_tx: mpsc::Sender<NetworkEvent>) -> Result<(), Box<dyn std::error::Error>> {
    // ローカル開発用
    let identity = Identity::self_signed(&["localhost", "127.0.0.1", "::1"]).unwrap();

    println!("{:?}", identity.certificate_chain().as_slice()[0].hash().fmt(wtransport::tls::Sha256DigestFmt::BytesArray));

    // WebTransportサーバーの設定
    let config = ServerConfig::builder()
        .with_bind_address(addr.parse()?)
        .with_identity(identity) // これ消えてる
        .build();
    
    let endpoint = Endpoint::server(config)?;
    println!("WebTransport server listening on https://{addr}");

    // クライアントからの接続待ちループ
    loop {
        let incoming_session = endpoint.accept().await;

        let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        let ecs_tx_clone = ecs_tx.clone();

        tokio::spawn(async move {
            // ハンドシェイク
            match incoming_session.await {
                Ok(session_request) => {
                    match session_request.accept().await {
                        Ok(connection) => {
                            println!("New WebTransport connection ID: {conn_id}");
                            wt_connection::handle_connection(connection, conn_id, ecs_tx_clone).await;
                        }
                        Err(e) => eprintln!("WebTransport session error: {e}"),
                    }
                }
                Err(e) => eprintln!("Incoming connection error: {e}"),
            }
        });
    }
}