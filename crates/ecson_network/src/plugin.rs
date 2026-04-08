//! ネットワーク系プラグイン（WebSocket / WebTransport）

use ecson_core::app::{EcsonApp, TokioRuntime, Update};
use ecson_core::plugin::Plugin;
use ecson_ecs::channels::NetworkEvent;
use ecson_ecs::events::{MessageReceived, MessageSendFailed, SendMessage, UserDisconnected};
use ecson_ecs::resources::{ConnectionMap, NetworkSender};
use ecson_ecs::systems::NetworkReceiver;
use tokio::sync::mpsc;
use tracing::error;

const DEFAULT_ECS_BUFFER: usize = 1024;
const DEFAULT_CLIENT_BUFFER: usize = 100;

/// ネットワーク共通の ECS リソース・イベント・システムをセットアップする。
///
/// 同一の World に対して複数のネットワークプラグインが `build` を呼んでも、
/// `ConnectionMap` の存在チェックにより二重登録を防ぐ。
fn setup_network_ecs(app: &mut EcsonApp, ecs_buffer: usize) {
    if app.world.contains_resource::<ConnectionMap>() {
        return;
    }

    let (ecs_tx, ecs_rx) = mpsc::channel::<NetworkEvent>(ecs_buffer);
    app.world.insert_resource(NetworkSender(ecs_tx));
    app.world.insert_resource(NetworkReceiver(ecs_rx));

    app.world.insert_resource(ConnectionMap::default());
    app.add_event::<MessageReceived>();
    app.add_event::<SendMessage>();
    app.add_event::<UserDisconnected>();
    app.add_event::<MessageSendFailed>();

    app.add_systems(
        Update,
        (
            ecson_ecs::systems::receive_network_messages_system,
            ecson_ecs::systems::flush_outbound_messages_system,
        ),
    );
}

/// `EcsonApp` に登録済みの `TokioRuntime` リソースを取得する。
///
/// # Panics
/// `TokioRuntime` が World に存在しない場合（= `EcsonApp::new()` を経由せずに
/// `EcsonApp` を構築した場合）パニックする。
fn get_runtime(app: &EcsonApp) -> TokioRuntime {
    app.world
        .get_resource::<TokioRuntime>()
        .expect("TokioRuntime が World に見つかりません。EcsonApp::new() で初期化してください。")
        .clone()
}

// ============================================================================
// WebSocket プラグイン
// ============================================================================

pub struct EcsonWebSocketPlugin {
    pub address: String,
    ecs_buffer: usize,
    client_buffer: usize,
}

impl EcsonWebSocketPlugin {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ecs_buffer: DEFAULT_ECS_BUFFER,
            client_buffer: DEFAULT_CLIENT_BUFFER,
        }
    }

    pub fn ecs_buffer(mut self, size: usize) -> Self {
        self.ecs_buffer = size;
        self
    }

    pub fn client_buffer(mut self, size: usize) -> Self {
        self.client_buffer = size;
        self
    }
}

impl Plugin for EcsonWebSocketPlugin {
    fn build(&self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        // 共有 Runtime 上にタスクをスポーンする。
        // std::thread::spawn + block_on を使わないため、スレッドが増えない。
        get_runtime(app).spawn(async move {
            if let Err(e) = crate::ws_server::run(&addr, ecs_tx, client_buffer).await {
                error!("Ecson WS Server Error: {e}");
            }
        });
    }
}

// ============================================================================
// WebSocket TLS（WSS）本番用プラグイン
// ============================================================================

pub struct EcsonWebSocketTlsPlugin {
    address: String,
    cert_path: String,
    key_path: String,
    ecs_buffer: usize,
    client_buffer: usize,
}

impl EcsonWebSocketTlsPlugin {
    pub fn new(
        address: impl Into<String>,
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            ecs_buffer: DEFAULT_ECS_BUFFER,
            client_buffer: DEFAULT_CLIENT_BUFFER,
        }
    }

    pub fn ecs_buffer(mut self, size: usize) -> Self {
        self.ecs_buffer = size;
        self
    }

    pub fn client_buffer(mut self, size: usize) -> Self {
        self.client_buffer = size;
        self
    }
}

impl Plugin for EcsonWebSocketTlsPlugin {
    fn build(&self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        // TLS アクセプタの構築は同期処理なので、spawn 前に済ませる。
        let acceptor = crate::tls::build_tls_acceptor(&self.cert_path, &self.key_path)
            .expect("TLS証明書の読み込みに失敗しました");

        get_runtime(app).spawn(async move {
            if let Err(e) = crate::wss_server::run(&addr, acceptor, ecs_tx, client_buffer).await {
                error!("Ecson WSS Server Error: {e}");
            }
        });
    }
}

// ============================================================================
// WebSocket TLS 開発用プラグイン（自己署名証明書を自動生成）
// ============================================================================

pub struct EcsonWebSocketTlsDevPlugin {
    address: String,
    ecs_buffer: usize,
    client_buffer: usize,
}

impl EcsonWebSocketTlsDevPlugin {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ecs_buffer: DEFAULT_ECS_BUFFER,
            client_buffer: DEFAULT_CLIENT_BUFFER,
        }
    }

    pub fn ecs_buffer(mut self, size: usize) -> Self {
        self.ecs_buffer = size;
        self
    }

    pub fn client_buffer(mut self, size: usize) -> Self {
        self.client_buffer = size;
        self
    }
}

impl Plugin for EcsonWebSocketTlsDevPlugin {
    fn build(&self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        let acceptor = crate::tls::build_self_signed_acceptor(vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ])
        .expect("自己署名証明書の生成に失敗しました");

        get_runtime(app).spawn(async move {
            if let Err(e) = crate::wss_server::run(&addr, acceptor, ecs_tx, client_buffer).await {
                error!("Ecson WSS Dev Server Error: {e}");
            }
        });
    }
}

// ============================================================================
// WebTransport 開発用プラグイン
// ============================================================================

pub struct EcsonWebTransportDevPlugin {
    pub address: String,
    ecs_buffer: usize,
    client_buffer: usize,
}

impl EcsonWebTransportDevPlugin {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ecs_buffer: DEFAULT_ECS_BUFFER,
            client_buffer: DEFAULT_CLIENT_BUFFER,
        }
    }

    pub fn ecs_buffer(mut self, size: usize) -> Self {
        self.ecs_buffer = size;
        self
    }

    pub fn client_buffer(mut self, size: usize) -> Self {
        self.client_buffer = size;
        self
    }
}

impl Plugin for EcsonWebTransportDevPlugin {
    fn build(&self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        get_runtime(app).spawn(async move {
            if let Err(e) = crate::wt_server::run(&addr, ecs_tx, client_buffer).await {
                error!("WebTransport Server Error: {e}");
            }
        });
    }
}
