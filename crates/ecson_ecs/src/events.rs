//! クライアントとの通信や、システム間でやり取りされるイベント（Message）を定義します。

use crate::channels::NetworkPayload;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::Message;

// =====================================================
// メッセージ送受信
// =====================================================

/// サーバーから特定のクライアントへメッセージを送信するためのイベント。
#[derive(Message)]
pub struct SendMessage {
    pub target: Entity,
    pub payload: NetworkPayload,
}

/// クライアントからメッセージを受信した際に発行されるイベント。
#[derive(Message)]
pub struct MessageReceived {
    pub entity: Entity,
    pub client_id: u64,
    pub payload: NetworkPayload,
}

// =====================================================
// クライアント接続・切断
// =====================================================

/// クライアントが接続した際に発行されるイベント
#[derive(Message)]
pub struct UserConnected {
    pub entity: Entity,
    pub client_id: u64,
}

/// クライアントとの接続が切断された際に発行されるイベント。
#[derive(Message)]
pub struct UserDisconnected {
    pub entity: Entity,
    pub client_id: u64,
}
