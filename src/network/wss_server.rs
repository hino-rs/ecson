//! TLS付きWebSocketサーバー（WSS）の起動と接続受け入れを管理するモジュール

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};

use crate::network::{channels::NetworkEvent, ws_connection};

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// TLS付きWebSocketサーバーを起動します。
///
/// # 引数
/// * `addr`          - リッスンするアドレス（例: "0.0.0.0:8443"）
/// * `acceptor`      - TLS設定済みの TlsAcceptor（`crate::network::tls` で構築）
/// * `ecs_tx`        - ECS側へのイベント送信チャンネル
/// * `client_buffer` - クライアント送信チャンネルのバッファサイズ
pub async fn run(
    addr: &str,
    acceptor: TlsAcceptor,
    ecs_tx: mpsc::Sender<NetworkEvent>,
    client_buffer: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket(TLS) server listening on wss://{addr}");

    while let Ok((tcp_stream, peer_addr)) = listener.accept().await {
        let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        let acceptor  = acceptor.clone();
        let ecs_tx    = ecs_tx.clone();

        info!("New TCP connection from: {peer_addr} (ID: {conn_id}), starting TLS handshake...");

        tokio::spawn(async move {
            // ── TLSハンドシェイク ──────────────────────────────────────────────
            // ここで証明書の提示・鍵交換が行われ、暗号化セッションが確立する。
            // 失敗した場合（証明書不一致・プロトコル非対応など）は接続を切る。
            let tls_stream = match acceptor.accept(tcp_stream).await {
                Ok(s)  => s,
                Err(e) => {
                    warn!("TLS handshake failed for ID {conn_id} ({peer_addr}): {e}");
                    return;
                }
            };

            info!("TLS handshake success for ID {conn_id}");

            // ── WebSocketハンドシェイク ───────────────────────────────────────
            // TlsStream は AsyncRead + AsyncWrite + Unpin を実装しているので
            // accept_async にそのまま渡せる。
            ws_connection::handle_connection(tls_stream, conn_id, ecs_tx, client_buffer).await;
        });
    }

    Ok(())
}