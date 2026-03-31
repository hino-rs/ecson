//! アプリケーション全体で共有されるECSリソースを定義します。
//! 状態の管理や、特定のデータへの高速なアクセス（O(1)ルックアップ）を提供します。

use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use crate::prelude::NetworkEvent;

/// ネットワーク接続ID（`u64`）から対応する `Entity` をO(1)で検索するための内部リソース。
/// Tokio側から送られてきたIDを元に、ECS側のエンティティを特定する際に使用します。
#[derive(Resource, Default)]
pub struct ConnectionMap(pub HashMap<u64, Entity>);

/// サーバーの回転に関するコンフィグ
#[derive(Resource, Clone)]
pub struct ServerTimeConfig {
    /// サーバーの目標Tickレート(Hz)
    pub tick_rate: f64,
    /// 1フレーム内で後れを取り戻すために実行できるFixedUpdateの最大回数
    pub max_ticks_per_frame: u32,
    /// 初理落ち時に警告ログを出すかどうか
    pub warn_on_lag: bool,
}

impl Default for ServerTimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60.0, 
            max_ticks_per_frame: 5, 
            warn_on_lag: false,
        }
        
    }
}

/// ルーム名（`String`）から、そのルームに参加している `Entity` の一覧をO(1)で検索するためのリソース。
/// 特定のルームに対するメッセージのブロードキャストなどを高速化します。
#[derive(Resource, Default)]
pub struct RoomMap(pub HashMap<String, HashSet<Entity>>);

/// ECSのWorldを経由して、Tokio側へイベントを送るためのチャンネル（Sender）を保持するリソース。
/// ネットワークプラグインの初期化時などに、Tokioランタイム側へクローンして渡すために使用されます。
#[derive(Resource)]
pub struct NetworkSender(pub mpsc::Sender<NetworkEvent>);