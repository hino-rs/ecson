use bevy_ecs::prelude::*;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use crate::network::channels::NetworkPayload;

// エンティティに持たせる送信用のポスト
#[derive(Component)]
pub struct ClientSender(pub mpsc::Sender<NetworkPayload>);

// エンティティに持たせるクライアントID
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

// クライアントが所属しているルームを表す
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Room(pub String);

