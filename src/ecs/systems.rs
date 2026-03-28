use crate::ecs::events::*;
use crate::network::channels::NetworkPayload;
use crate::{ecs::events::MessageReceived, network::channels::NetworkEvent};
use crate::ecs::components::*;
use bevy_ecs::{
    message::MessageWriter, resource::Resource, 
    system::{Commands, Query, ResMut}
};
use tokio::sync::mpsc;
use crate::ecs::resources::ConnectionMap;
use bevy_ecs::message::MessageReader;

// Resource用のラッパー構造体
#[derive(Resource)]
pub struct NetworkReceiver(pub mpsc::Receiver<NetworkEvent>);

// ネットワークからのイベントを処理する
pub fn receive_network_messages_system(
    mut commands: Commands,
    mut ecs_rx: ResMut<NetworkReceiver>,
    mut ev_msg: MessageWriter<MessageReceived>,
    mut ev_disconnect: MessageWriter<UserDisconnected>,
    mut connection_map: ResMut<ConnectionMap>,
) {
    while let Ok(event) = ecs_rx.0.try_recv() {
        match event {
            NetworkEvent::Connected { id, sender } => {
                let entity = commands.spawn((ClientId(id), ClientSender(sender))).id();
                connection_map.0.insert(id, entity);
                println!("ECS: 新規接続 {id} -> Entity {entity:?}");
            }
            NetworkEvent::Message { id, payload } => {
                if let Some(&entity) = connection_map.0.get(&id) {
                    ev_msg.write(MessageReceived { entity, client_id: id, payload });
                }
            }
            NetworkEvent::Disconnected { id } => {
                println!("ECS: {} が切断されました", id);
                if let Some(entity) = connection_map.0.remove(&id) {
                    ev_disconnect.write(UserDisconnected { entity, client_id: id });
                }
            }
        }
    }
}

// ECSから特定のクライアントにメッセージを送り返す
pub fn send_network_messages_system(
    query: Query<(&ClientId, &ClientSender)>,
) {
    for (client_id, sender) in query.iter() {
        let msg = NetworkPayload::Text("Hello from ECS Engine".into());

        // try_send を使って非同期待ちを回避
        if let Err(e) = sender.0.try_send(msg) {
            println!("{} への送信に失敗: {}", client_id.0, e);
        }
    }
}

pub fn flush_outbound_messages_system(
    mut outbound_messages: MessageReader<SendMessage>,
    query: Query<&ClientSender>,
) {
    for outbound in outbound_messages.read() {
        if let Ok(sender) = query.get(outbound.target) {
            if let Err(e) = sender.0.try_send(outbound.payload.clone()) {
                eprintln!("{e}");
            }
        } else {
            eprintln!("Destination Entity does not exist anymore");
        }
    }
}