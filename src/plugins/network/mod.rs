use crate::app::*;
use crate::ecs::events::UserDisconnected;
use crate::ecs::events::{MessageReceived, SendMessage};
use crate::ecs::resources::*;
use crate::ecs::systems::NetworkReceiver;
use crate::network::channels::NetworkEvent;
use crate::plugin::Plugin;
use tokio::sync::mpsc;
use tracing::{error, info};

// --------------------------------------------------------
// ネットワーク系共通処理
// --------------------------------------------------------

/// ネットワーク設定のデフォルト値
const DEFAULT_ECS_BUFFER: usize = 1024;
const DEFAULT_CLIENT_BUFFER: usize = 100;

/// ネットワーク系プラグイン（WebSocket / WebTransport）で共通して必要な
/// ECS側の初期化（チャンネル、リソース、システムの登録）を行います。
fn setup_network_ecs(app: &mut EcsonApp, ecs_buffer: usize) {
    // 既に初期化済みの場合はスキップ
    if app.world.contains_resource::<ConnectionMap>() {
        return;
    }

    // TokioとECSを繋ぐMPSCチャンネルを作成
    let (ecs_tx, ecs_rx) = mpsc::channel::<NetworkEvent>(ecs_buffer);
    app.world.insert_resource(NetworkSender(ecs_tx));
    app.world.insert_resource(NetworkReceiver(ecs_rx));

    // 初期化
    app.world.insert_resource(ConnectionMap::default());
    app.add_event::<MessageReceived>();
    app.add_event::<SendMessage>();
    app.add_event::<UserDisconnected>();

    // ネットワークメッセージの送受信・更新システムを登録
    app.add_systems(
        Update,
        (
            // ネットワークイベントの受信
            crate::ecs::systems::receive_network_messages_system,
            // ネットワークイベントの送信
            crate::ecs::systems::flush_outbound_messages_system,
        ),
    );
}

// --------------------------------------------------------
// WebSocket プラグイン
// --------------------------------------------------------

/// Tokioランタイムの起動からECSとのブリッジ構築までを隠蔽する、WebSocketサーバー用プラグイン。
pub struct EcsonWebSocketPlugin {
    pub address: String,
    /// ECS受信チャンネルのバッファサイズ(全クライアント共通)
    ecs_buffer: usize,
    /// クライアントごとの通知チャンネルのバッファサイズ
    client_buffer: usize,
}

impl EcsonWebSocketPlugin {
    /// 起動するアドレス(例: "127.0.0.1:8080")を指定してプラグインを生成します。
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ecs_buffer: DEFAULT_ECS_BUFFER,
            client_buffer: DEFAULT_CLIENT_BUFFER,
        }
    }

    /// ECS 受信チャンネルのバッファサイズを設定する。
    /// 接続数が多い・高頻度メッセージが来る場合は大きくする。
    pub fn ecs_buffer(mut self, size: usize) -> Self {
        self.ecs_buffer = size;
        self
    }

    /// クライアントごとの送信バッファサイズを設定する。
    /// サーバー側の送信が詰まりやすい場合は大きくする。
    pub fn client_buffer(mut self, size: usize) -> Self {
        self.client_buffer = size;
        self
    }
}

impl Plugin for EcsonWebSocketPlugin {
    fn build(self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        // Tokioランタイムをバックグラウンドスレッドで起動し、サーバーをリッスンさせる
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = crate::network::ws_server::run(&addr, ecs_tx, client_buffer).await {
                    error!("Ecson Server Error: {e}");
                }
            });
        });
    }
}

// --------------------------------------------------------
// WebTransport 開発用 プラグイン
// --------------------------------------------------------

/// Tokioランタイムの起動からECSとのブリッジ構築までを隠蔽する、WebTransportサーバー用プラグイン。
pub struct EcsonWebTransportDevPlugin {
    pub address: String,
    ecs_buffer: usize,
    client_buffer: usize,
}

impl EcsonWebTransportDevPlugin {
    /// 起動するアドレスを指定してプラグインを生成します。
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
    fn build(self, app: &mut EcsonApp) {
        setup_network_ecs(app, self.ecs_buffer);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();
        let client_buffer = self.client_buffer;

        // Tokioランタイムをバックグラウンドスレッドで起動し、WebTransportサーバーをリッスンさせる
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = crate::network::wt_server::run(&addr, ecs_tx, client_buffer).await {
                    error!("WebTransport Server Error: {e}");
                }
            });
        });
    }
}

// --------------------------------------------------------
// WebTransport 本番用 プラグイン
// --------------------------------------------------------

// pub struct EcsonWebTransportPlugin {
//     address: String,
//     identity: Identity,
//     ecs_buffer: usize,
//     client_buffer: usize,
// }

// impl EcsonWebTransportPlugin {
//     /// 証明書ファイルを指定する
//     pub fn new(address: impl Into<String>) -> Self {
//         todo!()
//     }

//     pub fn with_cert(
//         address: impl Into<String>,
//         cert_path: impl AsRef<Path>,
//         key_path: impl AsRef<Path>,
//     ) -> Result<>{
//     }
// }
