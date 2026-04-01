use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{RateLimitConfig, RateLimitState, RateLimitExceededEvent};

/// 受信メッセージのレートを検査し、超過した場合に制限を適用するシステム
///
/// Update スケジュールで動作し、ネットワーク受信の直後に実行されることを想定。
pub fn check_rate_limit_system(
    _config: Res<RateLimitConfig>,
    _query: Query<(Entity, &mut RateLimitState)>,
    _ev_received: MessageReader<MessageReceived>,
    _ev_exceeded: MessageWriter<RateLimitExceededEvent>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!(
        "受信メッセージを RateLimitState に記録し、\
        window_secs 内に max_messages を超えたら RateLimitAction に応じて Drop / Throttle / Disconnect を実行"
    )
}

/// 各クライアントのレートウィンドウを定期的にリセットするシステム
pub fn reset_rate_limit_windows_system(
    _config: Res<RateLimitConfig>,
    _query: Query<&mut RateLimitState>,
) {
    todo!("window_secs が経過したクライアントの message_count と window_start をリセット")
}
