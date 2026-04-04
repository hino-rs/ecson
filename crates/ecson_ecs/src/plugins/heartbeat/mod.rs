use crate::prelude::*;
use bevy_ecs::prelude::*;
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
    /// ping 識別メッセージ
    pub ping_payload: String,
    /// pong 識別メッセージ
    pub pong_payload: String,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10.0,
            timeout_secs: 30.0,
            ping_payload: "__ping__".into(),
            pong_payload: "__pong__".into(),
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
    last_pong_at: std::time::Instant,
    /// 未応答の Ping 数
    pending_pings: u32,
    /// 前回 Ping を送った時刻
    last_ping_at: std::time::Instant,
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self {
            last_pong_at: std::time::Instant::now(),
            pending_pings: 0,
            last_ping_at: std::time::Instant::now(),
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
    pub ping_payload: String,
    pub pong_payload: String,
}

impl Default for HeartbeatPlugin {
    fn default() -> Self {
        Self {
            interval_secs: 10.0,
            timeout_secs: 30.0,
            ping_payload: "__ping__".into(),
            pong_payload: "__pong__".into(),
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
            ping_payload: self.ping_payload,
            pong_payload: self.pong_payload,
        });

        app.add_event::<ClientTimedOutEvent>();

        app.add_systems(Update, (setup_heartbeat_system, receive_pong_system));
        app.add_systems(FixedUpdate, (send_ping_system, check_timeout_system));
    }
}
