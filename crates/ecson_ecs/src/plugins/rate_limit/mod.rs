use crate::prelude::*;
use bevy_ecs::prelude::*;
mod systems;
use systems::*;

// ============================================================================
// リソース
// ============================================================================

/// レート制限の設定
#[derive(Resource)]
pub struct RateLimitConfig {
    /// ウィンドウ期間（秒）
    pub window_secs: f32,
    /// ウィンドウ内の最大メッセージ数
    pub max_messages: u32,
    /// 超過時の動作
    pub action: RateLimitAction,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_secs: 1.0,
            max_messages: 30,
            action: RateLimitAction::Drop,
        }
    }
}

/// 超過時の動作
#[derive(Clone, Debug)]
pub enum RateLimitAction {
    /// メッセージを無視する
    Drop,
    /// クライアントを一時的に停止させる（メッセージを受け付けない）
    Throttle { duration_secs: f32 },
    /// クライアントを切断する
    Disconnect,
}

// ============================================================================
// コンポーネント
// ============================================================================

/// 各クライアントエンティティにアタッチされるレート追跡状態
#[derive(Component)]
pub struct RateLimitState {
    /// 現在のウィンドウ内のメッセージ数
    pub message_count: u32,
    /// 現在ウィンドウの開始時刻
    pub window_start: std::time::Instant,
    /// スロットル中の場合、解除時刻
    pub throttled_until: Option<std::time::Instant>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            message_count: 0,
            window_start: std::time::Instant::now(),
            throttled_until: None,
        }
    }
}

// ============================================================================
// イベント
// ============================================================================

/// クライアントがレート制限を超過したときに発火
#[derive(Message)]
pub struct RateLimitExceededEvent {
    pub entity: Entity,
    pub client_id: u64,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct RateLimitPlugin {
    pub window_secs: f32,
    pub max_messages: u32,
    pub action: RateLimitAction,
}

impl Default for RateLimitPlugin {
    fn default() -> Self {
        Self {
            window_secs: 1.0,
            max_messages: 30,
            action: RateLimitAction::Drop,
        }
    }
}

impl RateLimitPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn window(mut self, secs: f32) -> Self {
        self.window_secs = secs;
        self
    }

    pub fn max_messages(mut self, count: u32) -> Self {
        self.max_messages = count;
        self
    }

    pub fn on_exceed(mut self, action: RateLimitAction) -> Self {
        self.action = action;
        self
    }
}

impl Plugin for RateLimitPlugin {
    fn build(&self, app: &mut EcsonApp) {
        app.insert_resource(RateLimitConfig {
            window_secs: self.window_secs,
            max_messages: self.max_messages,
            action: self.action.clone(),
        });

        app.add_event::<RateLimitExceededEvent>();

        app.add_systems(Update, (setup_rate_limit_system, check_rate_limit_system));
        app.add_systems(FixedUpdate, reset_rate_limit_windows_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn default_state_has_zero_count() {
        let state = RateLimitState::default();
        assert_eq!(state.message_count, 0);
        assert!(state.throttled_until.is_none());
    }

    #[test]
    fn throttle_window_expires() {
        let state = RateLimitState {
            throttled_until: Some(Instant::now() - Duration::from_secs(1)),
            ..Default::default()
        };
        // 期限切れなら None として扱えることをシステム側ロジックと同じ条件で確認
        assert!(state.throttled_until.unwrap() < Instant::now());
    }
}
