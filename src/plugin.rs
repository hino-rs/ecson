//! プラグインシステムの中核となるトレイト群と、
//! チャットやネットワーク（WebSocket / WebTransport）関連の組み込みプラグインを定義します。

use crate::ecs::events::UserDisconnected;
use crate::app::{FluxionApp, MainSchedule};
use crate::ecs::events::{MessageReceived, SendMessage};
use crate::ecs::systems::NetworkReceiver;
use crate::network::channels::NetworkEvent;
use crate::ecs::resources::*;
use crate::prelude::ChatCommand;
use bevy_ecs::prelude::*;
use tokio::sync::mpsc;

/// プラグインのライフサイクル状態を表す列挙型。
/// （※将来的なステート管理のためのプレースホルダーです）
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    Adding,
    Ready,
    Finished,
    Cleaned,
}

// ============================================================================
// プラグインシステム基盤 (Traits & Macros)
// ============================================================================

/// 個別のプラグインが実装する基本トレイト。
pub trait Plugin {
    /// プラグインがアプリケーションに対してシステムやリソースを登録する処理を記述します。
    fn build(self, app: &mut FluxionApp);
}

/// `app.add_plugins()` に単一の `Plugin` や、複数の `Plugin` をまとめたタプルを渡せるようにするトレイト。
pub trait Plugins {
    /// 自身に含まれるプラグインをアプリケーションに追加します。
    fn add_to_app(self, app: &mut FluxionApp);
}

// 単一の Plugin を Plugins として扱えるようにするための実装
impl<P: Plugin> Plugins for P {
    fn add_to_app(self, app: &mut FluxionApp) {
        self.build(app);
    }
}

// タプルに対する Plugins の一括実装用マクロ
macro_rules! impl_plugins_for_tuples {
    ($($name:ident),*) => {
        impl<$($name: Plugin),*> Plugins for ($($name,)*) {
            #[allow(non_snake_case)]
            fn add_to_app(self, app: &mut FluxionApp) {
                let ($($name,)*) = self;
                $($name.build(app);)*
            }
        }
    };
}

// 最大6つのプラグインをタプルとして渡せるように実装を展開
impl_plugins_for_tuples!(P1);
impl_plugins_for_tuples!(P1, P2);
impl_plugins_for_tuples!(P1, P2, P3);
impl_plugins_for_tuples!(P1, P2, P3, P4);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6);


// ============================================================================
// 組み込みプラグイン
// ============================================================================

/// チャットアプリ開発用の基本リソースやイベントを登録するプラグイン。
pub struct ChatPlugin; 

impl Plugin for ChatPlugin {
    fn build(self, app: &mut FluxionApp) {
        app.world.insert_resource(RoomMap::default());
        app.add_event::<ChatCommand>();
        app.add_systems(MainSchedule, |mut msgs: ResMut<Messages<ChatCommand>>| {
            msgs.update()
        });
    }
}

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
    app.world.insert_resource(Messages::<MessageReceived>::default());
    app.world.insert_resource(Messages::<SendMessage>::default());

    // ネットワーク状態の管理リソース初期化
    app.world.insert_resource(ConnectionMap::default());
    app.world.insert_resource(RoomMap::default());
    app.add_event::<SendMessage>();
    app.add_event::<UserDisconnected>();

    // ネットワークメッセージの送受信・更新システムを登録
    app.add_systems(
        MainSchedule,
        (
            |mut msgs: ResMut<Messages<MessageReceived>>| msgs.update(),
            |mut msgs: ResMut<Messages<SendMessage>>| msgs.update(),
            |mut msgs: ResMut<Messages<UserDisconnected>>| msgs.update(),
            crate::ecs::systems::receive_network_messages_system,
            crate::ecs::systems::flush_outbound_messages_system,
            crate::ecs::systems::cleanup_disconnected_users_system, // あとで任意化
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