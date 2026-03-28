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

/// サーバーのメインループの更新頻度（Tickレート）を管理するリソース。
#[derive(Resource)]
pub struct ServerTickRate(pub f64);

impl Default for ServerTickRate {
    /// デフォルトは `NORMAL` (30.0Hz) に設定されます。
    fn default() -> Self {
        Self::NORMAL
    }
}

impl ServerTickRate {
    /// エコモード: 10Hz（処理負荷を最小限に抑えたい場合）
    pub const ECO: Self = Self(10.0);
    /// ハーフモード: 30Hz (ノーマルの半分)
    pub const HALF: Self = Self(30.0);
    /// ノーマルモード: 60Hz（一般的な状態同期）
    pub const NORMAL: Self = Self(60.0);
    /// 高速モード: 90Hz（アクション性が高い場合）
    pub const HIGH: Self = Self(90.0);
    /// リアルタイムモード: 120Hz（FPSや競技性の高いゲームなど）
    pub const REALTIME: Self = Self(120.0);
}

/// ルーム名（`String`）から、そのルームに参加している `Entity` の一覧をO(1)で検索するためのリソース。
/// 特定のルームに対するメッセージのブロードキャストなどを高速化します。
#[derive(Resource, Default)]
pub struct RoomMap(pub HashMap<String, HashSet<Entity>>);

/// ECSのWorldを経由して、Tokio側へイベントを送るためのチャンネル（Sender）を保持するリソース。
/// ネットワークプラグインの初期化時などに、Tokioランタイム側へクローンして渡すために使用されます。
#[derive(Resource)]
pub struct NetworkSender(pub mpsc::Sender<NetworkEvent>);