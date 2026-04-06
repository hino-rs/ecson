use crate::prelude::*;
use bevy_ecs::prelude::*;
mod systems;
use systems::*;

// ============================================================================
// リソース
// ============================================================================

/// スナップショットプラグインの設定
#[derive(Resource)]
pub struct SnapshotConfig {
    /// スナップショット送信間隔（秒）
    pub interval_secs: f32,
    /// 差分のみ送信するか（false の場合フル送信）
    pub delta_only: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            interval_secs: 0.1, // 10Hz
            delta_only: true,
        }
    }
}

/// 直前のスナップショット状態を保持するリソース
#[derive(Resource)]
pub struct SnapshotState {
    pub sequence: u64,
    pub last_snapshot: Vec<u8>,
    /// collect → broadcast 間のバッファ
    pub pending: Vec<u8>,
    /// インターバル計測用タイマー
    pub last_sent: std::time::Instant,
}

impl Default for SnapshotState {
    fn default() -> Self {
        Self {
            sequence: 0,
            last_snapshot: Vec::new(),
            pending: Vec::new(),
            // 起動直後に最初のスナップショットが即送信されるよう過去時刻で初期化
            last_sent: std::time::Instant::now() - std::time::Duration::from_secs(3600),
        }
    }
}

// ============================================================================
// コンポーネント
// ============================================================================

/// このコンポーネントがアタッチされたエンティティはスナップショットに含まれる
#[derive(Component)]
pub struct Snapshotable;

/// スナップショット送信先を絞り込むための購読コンポーネント
#[derive(Component)]
pub struct SnapshotSubscriber;

// ============================================================================
// イベント
// ============================================================================

/// スナップショットが生成・送信されたときに発火
#[derive(Message)]
pub struct SnapshotSentEvent {
    pub sequence: u64,
    pub delta: bool,
    pub byte_size: usize,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct SnapshotPlugin {
    pub interval_secs: f32,
    pub delta_only: bool,
}

impl Default for SnapshotPlugin {
    fn default() -> Self {
        Self {
            interval_secs: 0.1,
            delta_only: true,
        }
    }
}

impl SnapshotPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// スナップショット送信間隔を設定する（例: 20Hz なら `0.05`）
    pub fn interval(mut self, secs: f32) -> Self {
        self.interval_secs = secs;
        self
    }

    /// `true` にすると差分のみを送信する（帯域節約）
    pub fn delta_only(mut self, enabled: bool) -> Self {
        self.delta_only = enabled;
        self
    }
}

impl Plugin for SnapshotPlugin {
    fn build(&self, app: &mut EcsonApp) {
        app.world.insert_resource(SnapshotConfig {
            interval_secs: self.interval_secs,
            delta_only: self.delta_only,
        });
        app.world.insert_resource(SnapshotState::default());

        app.add_event::<SnapshotSentEvent>();

        app.add_systems(
            FixedUpdate,
            (collect_snapshot_system, broadcast_snapshot_system),
        );
    }
}
