use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{HeartbeatConfig, HeartbeatState, ClientTimedOutEvent};

/// 定期的にすべてのクライアントへ Ping を送信するシステム
pub fn send_ping_system(
    _config: Res<HeartbeatConfig>,
    _query: Query<(Entity, &mut HeartbeatState)>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("一定間隔で各クライアントに Ping メッセージを送信し、pending_pings をインクリメント")
}

/// クライアントから届いた Pong を処理するシステム
pub fn receive_pong_system(
    _ev_received: MessageReader<MessageReceived>,
    _query: Query<&mut HeartbeatState>,
) {
    todo!("'pong' テキストを受信したら last_pong_at を更新し、pending_pings をリセット")
}

/// タイムアウトしたクライアントを検出して切断するシステム
pub fn check_timeout_system(
    _config: Res<HeartbeatConfig>,
    _query: Query<(Entity, &HeartbeatState)>,
    _ev_timeout: MessageWriter<ClientTimedOutEvent>,
) {
    todo!("last_pong_at が timeout_secs を超えたクライアントを検出し、ClientTimedOutEvent を発行して切断")
}
