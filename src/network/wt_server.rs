//! WebTransportサーバーの起動と、クライアント接続の受け入れを管理するモジュールです。

use tokio::sync::mpsc;
use wtransport::Identity;
use wtransport::{Endpoint, ServerConfig};
use crate::network::channels::NetworkEvent;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::network::wt_connection;

/// クライアント接続ごとに一意のIDを生成するための、スレッドセーフなカウンター。
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// WebTransportサーバーを起動し、クライアントからの接続を非同期で待機します。
/// 
/// 開発用の自己署名証明書を自動生成し、エンドポイントをバインドします。
/// 接続要求を受け取ると新しいタスクを生成し、WebTransport特有のハンドシェイクを経て
/// コネクション処理へ委譲します。
/// 
/// # 引数
/// * `addr` - サーバーがリッスンするアドレス（例: "127.0.0.1:4433"）
/// * `ecs_tx` - Tokio側のネットワークイベントをECS側へ伝達するための送信チャンネル
pub async fn run(addr: &str, ecs_tx: mpsc::Sender<NetworkEvent>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. ローカル開発用の自己署名証明書（Self-signed Certificate）を生成
    let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"]).unwrap();

    // クライアント側で証明書の検証をパス（またはハッシュ照合）するために必要なハッシュ値を出力
    println!(
        "Certificate Hash: {:?}", 
        identity.certificate_chain().as_slice()[0].hash().fmt(wtransport::tls::Sha256DigestFmt::BytesArray)
    );

    // 2. WebTransportサーバーの設定
    let config = ServerConfig::builder()
        .with_bind_address(addr.parse()?)
        .with_identity(identity) 
        .build();
    
    // エンドポイントをバインドしてリッスン開始
    let endpoint = Endpoint::server(config)?;
    println!("WebTransport server listening on https://{addr}");

    // 3. クライアントからの接続待ちループ
    loop {
        // 新しい接続要求（IncomingSession）を待機
        let incoming_session = endpoint.accept().await;

        // 新しい接続が来るたびにIDを1進めて取得
        let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        let ecs_tx_clone = ecs_tx.clone();

        // 接続ごとのハンドシェイクと通信処理を非同期タスクとして分離
        tokio::spawn(async move {
            // WebTransportのハンドシェイクプロセス:
            // (1) セッション要求の待機
            match incoming_session.await {
                Ok(session_request) => {
                    // (2) セッション要求の受諾
                    match session_request.accept().await {
                        Ok(connection) => {
                            println!("New WebTransport connection established (ID: {conn_id})");
                            
                            // (3) 確立されたコネクションの処理を `wt_connection::handle_connection` へ委譲
                            wt_connection::handle_connection(connection, conn_id, ecs_tx_clone).await;
                        }
                        Err(e) => eprintln!("WebTransport session accept error for ID {conn_id}: {e}"),
                    }
                }
                Err(e) => eprintln!("Incoming WebTransport connection error: {e}"),
            }
        });
    }
}