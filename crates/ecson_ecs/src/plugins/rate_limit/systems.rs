use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{RateLimitAction, RateLimitConfig, RateLimitExceededEvent, RateLimitState};

/// 新しく接続したクライアントに RateLimitState を付与するシステム
///
/// heartbeat の setup_heartbeat_system と同じパターン。
/// ClientId を持つが RateLimitState をまだ持っていないエンティティへアタッチする。
pub fn setup_rate_limit_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<RateLimitState>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(RateLimitState::default());
    }
}

/// 受信メッセージのレートを検査し、超過した場合に制限を適用するシステム
///
/// # 動作
/// 1. スロットル中のクライアントからのメッセージは即 Drop し `RateLimitExceededEvent` を発行
/// 2. ウィンドウ内カウントが `max_messages` を超えたら `RateLimitAction` に応じて処理
///    - `Drop`       : イベントを発行するだけ（他システムへの伝播は防げない点に注意）
///    - `Throttle`   : `throttled_until` をセットし警告を送信
///    - `Disconnect` : 警告を送信 → `UserDisconnected` 発行 → despawn
///
/// # スケジュール: Update（ネットワーク受信直後に動作）
pub fn check_rate_limit_system(
    mut commands: Commands,
    config: Res<RateLimitConfig>,
    mut query: Query<(Entity, &mut RateLimitState, &ClientId)>,
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_exceeded: MessageWriter<RateLimitExceededEvent>,
    mut ev_send: MessageWriter<SendMessage>,
    mut ev_disconnect: MessageWriter<UserDisconnected>,
) {
    for msg in ev_received.read() {
        let Ok((entity, mut state, client_id)) = query.get_mut(msg.entity) else {
            continue;
        };
        let now = std::time::Instant::now();

        // ── スロットル中チェック ──────────────────────────────────────────
        if let Some(until) = state.throttled_until {
            if now < until {
                // 期間内はカウントせず破棄
                ev_exceeded.write(RateLimitExceededEvent {
                    entity,
                    client_id: client_id.0,
                });
                continue;
            }
            // 期間が過ぎていたら解除
            state.throttled_until = None;
        }

        // ── カウントアップ ────────────────────────────────────────────────
        state.message_count += 1;

        if state.message_count <= config.max_messages {
            continue; // 制限内なので何もしない
        }

        // ── 制限超過 ──────────────────────────────────────────────────────
        ev_exceeded.write(RateLimitExceededEvent {
            entity,
            client_id: client_id.0,
        });

        match &config.action {
            RateLimitAction::Drop => {
                // このシステムとしての処理はここで終了。
                // 注: 他システムが同一の MessageReceived を独立して読むため
                //     完全なドロップは現アーキテクチャでは行えない。
            }

            RateLimitAction::Throttle { duration_secs } => {
                state.throttled_until =
                    Some(now + std::time::Duration::from_secs_f32(*duration_secs));
                ev_send.write(SendMessage {
                    target: entity,
                    payload: NetworkPayload::Text(
                        "[RateLimit] Too many messages. You are temporarily throttled.".to_string(),
                    ),
                });
            }

            RateLimitAction::Disconnect => {
                ev_send.write(SendMessage {
                    target: entity,
                    payload: NetworkPayload::Text(
                        "[RateLimit] Disconnected due to rate limit violation.".to_string(),
                    ),
                });
                ev_disconnect.write(UserDisconnected {
                    entity,
                    client_id: client_id.0,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

/// 各クライアントのレートウィンドウを定期的にリセットするシステム
///
/// `window_secs` が経過したクライアントの `message_count` と `window_start` をリセットする。
///
/// # スケジュール: FixedUpdate
pub fn reset_rate_limit_windows_system(
    config: Res<RateLimitConfig>,
    mut query: Query<&mut RateLimitState>,
) {
    let now = std::time::Instant::now();
    for mut state in query.iter_mut() {
        if now.duration_since(state.window_start).as_secs_f32() >= config.window_secs {
            state.message_count = 0;
            state.window_start = now;
        }
    }
}