use super::{ClientTimedOutEvent, HeartbeatConfig, HeartbeatState};
use crate::prelude::*;
use bevy_ecs::prelude::*;

/// 定期的にすべてのクライアントへ Ping を送信するシステム
pub fn send_ping_system(
    config: Res<HeartbeatConfig>,
    mut query: Query<(Entity, &mut HeartbeatState)>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for (entity, mut state) in query.iter_mut() {
        if state.last_ping_at.elapsed().as_secs_f32() >= config.interval_secs {
            ev_send.write(SendMessage {
                target: entity,
                payload: NetworkPayload::Text("ping".to_string()),
            });

            state.pending_pings += 1;
            state.last_ping_at = std::time::Instant::now();
        }
    }
}

/// クライアントから届いた Pong を処理するシステム
pub fn receive_pong_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut query: Query<&mut HeartbeatState>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else {
            continue;
        };
        if text.trim() != "pong" {
            continue;
        }

        if let Ok(mut state) = query.get_mut(msg.entity) {
            state.last_pong_at = std::time::Instant::now();
            state.pending_pings = 0;
        }
    }
}

/// タイムアウトしたクライアントを検出して切断するシステム
pub fn check_timeout_system(
    mut commands: Commands,
    config: Res<HeartbeatConfig>,
    query: Query<(Entity, &HeartbeatState, &ClientId)>,
    mut ev_timeout: MessageWriter<ClientTimedOutEvent>,
    mut ev_disconnect: MessageWriter<UserDisconnected>,
) {
    for (entity, state, client_id) in query.iter() {
        if state.last_pong_at.elapsed().as_secs_f32() > config.timeout_secs {
            ev_timeout.write(ClientTimedOutEvent { entity });
            ev_disconnect.write(UserDisconnected {
                entity,
                client_id: client_id.0,
            });
            commands.entity(entity).despawn();
        }
    }
}

/// ClientIdを持っているエンティティにHeartbeatStateをアタッチする
pub fn setup_heartbeat_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<HeartbeatState>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(HeartbeatState::default());
    }
}
