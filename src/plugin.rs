use crate::app::{FluxionApp, MainSchedule};
use crate::ecs::events::MessageReceived;
use crate::ecs::systems::{NetworkReceiver, receive_network_messages_system};
use crate::network::channels::NetworkEvent;
use crate::prelude::ConnectionMap;
use bevy_ecs::prelude::*;
use tokio::sync::mpsc;
use crate::server;

/// Tokioランタイムの起動からECSとのブリッジ構築までを隠蔽するプラグイン
pub struct FluxionNetworkPlugin {
    pub address: String,
}

impl FluxionNetworkPlugin {
    /// 起動するアドレス(例: "127.0.0.1:8080")を指定してプラグイン生成
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }
}

impl Plugin for FluxionNetworkPlugin {
    fn build(self, app: &mut FluxionApp) {
        // TokioとECSを繋ぐMPSCチャンネルを作成
        let (ecs_tx, ecs_rx) = mpsc::channel::<NetworkEvent>(1024);

        // Tokioランタイムをバックグラウンドスレッドで起動し、サーバーをリッスンさせる
        let addr = self.address.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = server::run(&addr, ecs_tx).await {
                    eprintln!("Fluxion Server Error: {e}");
                }
            });
        });

        // ECSリソースの登録
        app.world.insert_resource(NetworkReceiver(ecs_rx));
        app.world.insert_resource(Messages::<MessageReceived>::default());
        app.world.insert_resource(ConnectionMap::default());

        // 必須システムの登録
        app.add_systems(
            MainSchedule,
            (
                |mut msgs: ResMut<Messages<MessageReceived>>| msgs.update(), // 古いイベントの破棄
                receive_network_messages_system,                             // 受信イベントの橋渡し
            )
        );
    }
}

/// 個別のプラグインが実装する基本トレイト
pub trait Plugin {
    fn build(self, app: &mut FluxionApp);
}

/// app.add_plugins()に単一のPluginや複数のPluginをタプルで渡せるようにするトレイト
pub trait Plugins {
    fn add_to_app(self, app: &mut FluxionApp);
}

// 単一のPluginをPluginsとして扱えるようにする
impl<P: Plugin> Plugins for P {
    fn add_to_app(self, app: &mut FluxionApp) {
        self.build(app);
    }
}

// タプルに対するPluginsの実装
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

impl_plugins_for_tuples!(P1);
impl_plugins_for_tuples!(P1, P2);
impl_plugins_for_tuples!(P1, P2, P3);
impl_plugins_for_tuples!(P1, P2, P3, P4);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6);

#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    Adding,
    Ready,
    Finished,
    Cleaned,
}

// Fluxionが動くためのコア機能を一手に引き受けるプラグイン
pub struct FluxionCorePlugin {
    pub ecs_rx: mpsc::Receiver<NetworkEvent>,
}

impl Plugin for FluxionCorePlugin {
    fn build(self, app: &mut FluxionApp) {
        
        // リソースの登録
        app.world.insert_resource(NetworkReceiver(self.ecs_rx));
        app.world.insert_resource(Messages::<MessageReceived>::default());


        // コアシステムの登録
        app.add_systems(
            MainSchedule,
            (
                // 古いメッセージイベントの破棄
                |mut msgs: ResMut<Messages<MessageReceived>>| msgs.update(),
                // ネットワークからECSへのメッセージの橋渡し
                receive_network_messages_system,
            ),
        );
    }
}
