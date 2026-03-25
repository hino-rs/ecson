use crate::{ecs::events::MessageReceived, network::channels::NetworkEvent};
use crate::ecs::components::*;
use bevy_ecs::entity;
use bevy_ecs::{
    entity::Entity, message::MessageWriter, resource::Resource, 
    system::{Commands, Query, ResMut}
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use crate::ecs::components::ConnectionMap;

// Resource用のラッパー構造体
#[derive(Resource)]
pub struct NetworkReceiver(pub mpsc::Receiver<NetworkEvent>);

// ネットワークからのイベントを処理する
pub fn receive_network_messages_system(
    mut commands: Commands,
    mut ecs_rx: ResMut<NetworkReceiver>,
    mut ev_msg: MessageWriter<MessageReceived>,
    // quert: Query<(Entity, &ClientId)>,
    mut connection_map: ResMut<ConnectionMap>,
) {
    while let Ok(event) = ecs_rx.0.try_recv() {
        match event {
            NetworkEvent::Connected { id, sender } => {
                
                let entity = commands.spawn((ClientId(id), ClientSender(sender))).id();
                connection_map.0.insert(id, entity);
                println!("ECS: 新規接続 {id} -> Entity {entity:?}");

                // クライアントをエンティティとしてWorldに召喚
                // commands.spawn((
                //     ClientId(id),
                //     ClientSender(sender),
                // ));
            }
            NetworkEvent::Message { id, msg } => {
                if let Some(&entity) = connection_map.0.get(&id) {
                    ev_msg.write(MessageReceived { entity, client_id: id, msg });
                }
                // ev_msg.write(MessageReceived { client_id: id, msg });
            }
            NetworkEvent::Disconnected { id } => {
                println!("ECS: {} が切断されました", id);
                // （本当はここでエンティティをDespawnして削除する処理を書く）
            }
        }
    }
}

// ECSから特定のクライアントにメッセージを送り返す
pub fn send_network_messages_system(
    query: Query<(&ClientId, &ClientSender)>,
) {
    for (client_id, sender) in query.iter() {
        let msg = Message::Text("Hello from ECS Engine".into());

        // try_send を使って非同期待ちを回避
        if let Err(e) = sender.0.try_send(msg) {
            // 送信キューが満杯の場合などのエラーハンドリング
            // println!("{} への送信に失敗: {}", client_id.0, e);
        }
    }
}