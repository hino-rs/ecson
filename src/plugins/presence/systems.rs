use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{PresenceMap, PresenceStatus, PresenceChangedEvent};

/// 受信メッセージから在席ステータス変更コマンドを解析するシステム
pub fn parse_presence_messages_system(
    _ev_received: MessageReader<MessageReceived>,
    _ev_presence: MessageWriter<PresenceChangedEvent>,
) {
    todo!("'/status online|away|busy' 等のメッセージを解析し、PresenceChangedEvent を発行")
}

/// PresenceChangedEvent を処理して PresenceMap と Component を更新するシステム
pub fn handle_presence_update_system(
    _ev_presence: MessageReader<PresenceChangedEvent>,
    _presence_map: ResMut<PresenceMap>,
    _query: Query<&mut PresenceStatus>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("PresenceMap と PresenceStatus コンポーネントを更新し、必要に応じて他クライアントへ通知")
}

/// クライアント切断時に PresenceMap からエントリを削除するシステム
pub fn handle_presence_disconnect_system(
    _ev_disconnected: MessageReader<crate::ecs::events::UserDisconnected>,
    _presence_map: ResMut<PresenceMap>,
) {
    todo!("UserDisconnected を受け取り、PresenceMap から該当クライアントを削除")
}
