//! クライアントとの通信や、システム間でやり取りされるイベント（メッセージ）を定義します。

use bevy_ecs::{entity::Entity, event::Event};
use bevy_ecs::message::Message;
use crate::channels::NetworkPayload;

/// クライアントからメッセージを受信した際に発行されるイベント。
#[derive(Event, Message)]
pub struct MessageReceived {
    pub entity: Entity,
    pub client_id: u64,
    pub payload: NetworkPayload,
}

/// サーバーから特定のクライアントへメッセージを送信するためのイベント。
#[derive(Message)]
pub struct SendMessage {
    pub target: Entity,
    pub payload: NetworkPayload,
}

/// クライアントとの接続が切断された際に発行されるイベント。
#[derive(Event, Message)]
pub struct UserDisconnected {
    pub entity: Entity,
    pub client_id: u64,
}
