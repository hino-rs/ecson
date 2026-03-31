use crate::ecs::events::UserDisconnected;
use crate::app::*;
use crate::ecs::events::{MessageReceived, SendMessage};
use crate::ecs::systems::NetworkReceiver;
use crate::network::channels::NetworkEvent;
use crate::ecs::resources::*;
use tokio::sync::mpsc;
use crate::plugin::Plugin;

// --------------------------------------------------------
// ネットワーク系共通処理
// --------------------------------------------------------

/// ネットワーク系プラグイン（WebSocket / WebTransport）で共通して必要な
/// ECS側の初期化（チャンネル、リソース、システムの登録）を行います。
fn setup_network_ecs(app: &mut FluxionApp) {
    // 既に初期化済みの場合はスキップ
    if app.world.contains_resource::<ConnectionMap>() {
        return;
    }

    // TokioとECSを繋ぐMPSCチャンネルを作成
    let (ecs_tx, ecs_rx) = mpsc::channel::<NetworkEvent>(1024);
    app.world.insert_resource(NetworkSender(ecs_tx));
    app.world.insert_resource(NetworkReceiver(ecs_rx));

    // メッセージ系イベントリソースの初期化
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
            // 切断されたユーザーのクリーンアップ（先ほど追加したもの）
            crate::ecs::systems::cleanup_disconnected_users_system, // 後で任意化
            // ネットワークイベントの送信
            crate::ecs::systems::flush_outbound_messages_system,
        )
    );
}



// --------------------------------------------------------
// WebSocket プラグイン
// --------------------------------------------------------

/// Tokioランタイムの起動からECSとのブリッジ構築までを隠蔽する、WebSocketサーバー用プラグイン。
pub struct FluxionWebSocketPlugin {
    pub address: String,
}

impl FluxionWebSocketPlugin {
    /// 起動するアドレス(例: "127.0.0.1:8080")を指定してプラグインを生成します。
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }
}

impl Plugin for FluxionWebSocketPlugin {
    fn build(self, app: &mut FluxionApp) {
        setup_network_ecs(app);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();

        // Tokioランタイムをバックグラウンドスレッドで起動し、サーバーをリッスンさせる
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = crate::server::run(&addr, ecs_tx).await {
                    eprintln!("Fluxion Server Error: {e}");
                }
            });
        });
    }
}

// --------------------------------------------------------
// WebTransport プラグイン
// --------------------------------------------------------

/// Tokioランタイムの起動からECSとのブリッジ構築までを隠蔽する、WebTransportサーバー用プラグイン。
pub struct FluxionWebTransportPlugin {
    pub address: String,
}

impl FluxionWebTransportPlugin {
    /// 起動するアドレスを指定してプラグインを生成します。
    pub fn new(address: impl Into<String>) -> Self {
        Self { address: address.into() }
    }
}

impl Plugin for FluxionWebTransportPlugin {
    fn build(self, app: &mut FluxionApp) {
        setup_network_ecs(app);

        let ecs_tx = app.world.get_resource::<NetworkSender>().unwrap().0.clone();
        let addr = self.address.clone();

        // Tokioランタイムをバックグラウンドスレッドで起動し、WebTransportサーバーをリッスンさせる
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = crate::network::wt_server::run(&addr, ecs_tx).await {
                    eprintln!("WebTransport Server Error: {e}");
                }
            });
        });
    }
}

