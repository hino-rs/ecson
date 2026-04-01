use bevy_ecs::prelude::*;
use crate::prelude::*;
mod systems;
use systems::*;

// ============================================================================
// リソース
// ============================================================================

/// ハートビート設定
#[derive(Resource)]
pub struct HeartbeatConfig {
    /// Ping 送信間隔（秒）
    pub interval_secs: f32,
    /// この秒数以内に Pong が返らなければ切断とみなす
    pub timeout_secs: f32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10.0,
            timeout_secs: 30.0,
        }
    }
}

// ============================================================================
// コンポーネント
// ============================================================================

/// 各クライアントエンティティにアタッチされるハートビート状態
#[derive(Component)]
pub struct HeartbeatState {
    /// 最後に Pong を受け取った（または接続した）時刻
    pub last_pong_at: std::time::Instant,
    /// 未応答の Ping 数
    pub pending_pings: u32,
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self {
            last_pong_at: std::time::Instant::now(),
            pending_pings: 0,
        }
    }
}

// ============================================================================
// イベント
// ============================================================================

/// クライアントがタイムアウトにより切断されたときに発火
#[derive(Message)]
pub struct ClientTimedOutEvent {
    pub entity: Entity,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct HeartbeatPlugin {
    pub interval_secs: f32,
    pub timeout_secs: f32,
}

impl Default for HeartbeatPlugin {
    fn default() -> Self {
        Self {
            interval_secs: 10.0,
            timeout_secs: 30.0,
        }
    }
}

impl HeartbeatPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interval(mut self, secs: f32) -> Self {
        self.interval_secs = secs;
        self
    }

    pub fn timeout(mut self, secs: f32) -> Self {
        self.timeout_secs = secs;
        self
    }
}

impl Plugin for HeartbeatPlugin {
    fn build(self, app: &mut EcsonApp) {
        app.world.insert_resource(HeartbeatConfig {
            interval_secs: self.interval_secs,
            timeout_secs: self.timeout_secs,
        });

        app.add_event::<ClientTimedOutEvent>();

        app.add_systems(Update, receive_pong_system);
        app.add_systems(
            FixedUpdate,
            (
                send_ping_system,
                check_timeout_system,
            ),
        );
    }
}
