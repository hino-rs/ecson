use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{PresenceMap, PresenceStatus, PresenceChangedEvent};

/// 受信メッセージから在席ステータス変更コマンドを解析するシステム
pub fn parse_presence_messages_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_presence: MessageWriter<PresenceChangedEvent>,
) {
    for msg in ev_received.read() {
        if let NetworkPayload::Text(text) = &msg.payload {
            let status = match text.trim() {
                "/status online" => PresenceStatus::Online,
                "/status away"   => PresenceStatus::Away,
                "/status busy"   => PresenceStatus::Busy,
                _                => continue,
            };
            ev_presence.write(PresenceChangedEvent {
                client_id: msg.client_id,
                entity: msg.entity,
                status
            });
        }
    }
}

/// PresenceChangedEvent を処理して PresenceMap と Component を更新するシステム
pub fn handle_presence_update_system(
    mut ev_presence: MessageReader<PresenceChangedEvent>,
    mut presence_map: ResMut<PresenceMap>,
    mut query: Query<&mut PresenceStatus>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<Entity, With<ClientId>>,
) {
    for ev in ev_presence.read() {
        presence_map.map.insert(ev.client_id, ev.status.clone());
        if let Ok(mut status) = query.get_mut(ev.entity) {
            *status = ev.status.clone();
        }
        for target_entity in client_query.iter() {
            ev_send.write(SendMessage {
                target: target_entity,
                payload: NetworkPayload::Text(format!("{} {}", ev.client_id, ev.status)),
            });
        }
    }

}

/// クライアント切断時に PresenceMap からエントリを削除するシステム
pub fn handle_presence_disconnect_system(
    mut ev_disconnected: MessageReader<crate::events::UserDisconnected>,
    mut presence_map: ResMut<PresenceMap>,
) {
    for ev in ev_disconnected.read() {
        presence_map.map.remove(&ev.client_id);
    }
}
