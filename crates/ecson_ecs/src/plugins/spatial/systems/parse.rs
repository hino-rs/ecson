use crate::plugins::spatial::events::{ClientMovedEvent, parse_move_command};
use crate::prelude::*;
use bevy_ecs::prelude::*;

/// 受信メッセージから "/move x y [z]" を解析し、ClientMovedEvent を発行する
///
/// - "/move x y"     → MovePayload::Move2D
/// - "/move x y z"   → MovePayload::Move3D
/// - パース失敗 / 関係ないメッセージ → スキップ
///
/// Schedule: Update（受信即時パース）
pub fn parse_move_messages_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_moved: MessageWriter<ClientMovedEvent>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else {
            continue;
        };

        let Some(payload) = parse_move_command(text) else {
            continue;
        };

        ev_moved.write(ClientMovedEvent {
            entity: msg.entity,
            client_id: msg.client_id,
            payload,
        });
    }
}
