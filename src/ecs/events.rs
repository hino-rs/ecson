use bevy_ecs::{entity::Entity, event::Event};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use bevy_ecs::message::Message;
use crate::network::channels::NetworkPayload;

#[derive(Event, Message)]
pub struct MessageReceived {
    pub entity: Entity,
    pub client_id: u64,
    pub payload: NetworkPayload,
}

// ユーザーが送信要素を出すためのイベント
#[derive(Message)]
pub struct SendMessage {
    pub target: Entity,
    pub payload: NetworkPayload,
}

// 上の全体に一斉送信版
#[derive(Message)]
pub struct BroadcastMessage {
    pub msg: NetworkPayload,
}