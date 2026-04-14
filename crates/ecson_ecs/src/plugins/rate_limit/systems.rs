use super::{RateLimitAction, RateLimitConfig, RateLimitExceededEvent, RateLimitState};
use crate::prelude::*;
use bevy_ecs::prelude::*;

/// System that attaches `RateLimitState` to newly connected clients.
///
/// Follows the same pattern as `setup_heartbeat_system`.
/// Attaches to entities that have `ClientId` but not yet `RateLimitState`.
pub fn setup_rate_limit_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<RateLimitState>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(RateLimitState::default());
    }
}

/// System that inspects incoming message rates and applies limits when exceeded.
///
/// # Behavior
/// 1. Messages from throttled clients are immediately dropped and a `RateLimitExceededEvent` is fired.
/// 2. When the count within the window exceeds `max_messages`, the configured `RateLimitAction` is applied:
///    - `Drop`       : fires the event only (note: cannot prevent other systems from reading the same `MessageReceived`)
///    - `Throttle`   : sets `throttled_until` and sends a warning to the client
///    - `Disconnect` : sends a warning, fires `UserDisconnected`, then despawns the entity
///
/// # Schedule: Update (runs immediately after network receive)
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

        // ── throttle check ───────────────────────────────────────────────
        if let Some(until) = state.throttled_until {
            if now < until {
                // still within throttle window — drop without counting
                ev_exceeded.write(RateLimitExceededEvent {
                    entity,
                    client_id: client_id.0,
                });
                continue;
            }
            // window expired — clear throttle
            state.throttled_until = None;
        }

        // ── count up ─────────────────────────────────────────────────────
        state.message_count += 1;

        if state.message_count <= config.max_messages {
            continue; // within limit, nothing to do
        }

        // ── limit exceeded ───────────────────────────────────────────────
        ev_exceeded.write(RateLimitExceededEvent {
            entity,
            client_id: client_id.0,
        });

        match &config.action {
            RateLimitAction::Drop => {
                // This system's handling ends here.
                // Note: a full drop is not possible in the current architecture because
                // other systems read the same MessageReceived independently.
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

/// System that periodically resets the rate window for each client.
///
/// Resets `message_count` and `window_start` for clients whose `window_secs` have elapsed.
///
/// # Schedule: FixedUpdate
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
