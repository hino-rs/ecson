//! ネットワークイベントの受信処理や、クライアントへのメッセージ送信を行うECSシステム群です。

use crate::channels::NetworkEvent;
use crate::components::*;
use crate::events::*;
use crate::resources::ConnectionMap;
use bevy_ecs::message::MessageReader;
use bevy_ecs::{
    message::MessageWriter,
    resource::Resource,
    system::{Commands, Query, ResMut},
};
use tokio::sync::mpsc;
use tracing::warn;
use tracing::{error, info};

/// Tokio側から送られてくるネットワークイベントを受信するためのリソースラッパー。
#[derive(Resource)]
pub struct NetworkReceiver(pub mpsc::Receiver<NetworkEvent>);

/// ネットワーク層からのイベントをポーリングし、ECS側の状態を更新するシステム。
pub fn receive_network_messages_system(
    mut commands: Commands,
    mut ecs_rx: ResMut<NetworkReceiver>,
    mut connection_map: ResMut<ConnectionMap>,
    mut ev_msg: MessageWriter<MessageReceived>,
    mut ev_connected: MessageWriter<UserConnected>,
    mut ev_disconnect: MessageWriter<UserDisconnected>,
) {
    while let Ok(event) = ecs_rx.0.try_recv() {
        match event {
            NetworkEvent::Connected { id, sender } => {
                if connection_map.0.contains_key(&id) {
                    warn!("conn_id collision: {id}, dropping");
                    return;
                }
                let entity = commands.spawn((ClientId(id), ClientSender(sender))).id();
                connection_map.0.insert(id, entity);
                ev_connected.write(UserConnected {
                    entity,
                    client_id: id,
                });
                info!("ECS: 新規接続 {id} -> Entity {entity:?}");
            }
            NetworkEvent::Message { id, payload } => {
                if let Some(&entity) = connection_map.0.get(&id) {
                    ev_msg.write(MessageReceived {
                        entity,
                        client_id: id,
                        payload,
                    });
                }
            }
            NetworkEvent::Disconnected { id } => {
                info!("ECS: {} が切断されました", id);

                if let Some(entity) = connection_map.0.remove(&id) {
                    ev_disconnect.write(UserDisconnected {
                        entity,
                        client_id: id,
                    });
                }
            }
        }
    }
}

/// ECS内で発行された送信要求（`SendMessage`）を処理し、ネットワーク層へ引き渡すシステム。
pub fn flush_outbound_messages_system(
    mut outbound_messages: MessageReader<SendMessage>,
    query: Query<&ClientSender>,
    mut ev_failed: MessageWriter<MessageSendFailed>,
) {
    for outbound in outbound_messages.read() {
        if let Ok(sender) = query.get(outbound.target) {
            if let Err(e) = sender.0.try_send(outbound.payload.clone()) {
                error!(
                    "Failed to send message to Entity {:?}: {e}",
                    outbound.target
                );
                ev_failed.write(MessageSendFailed {
                    entity: outbound.target,
                    reason: SendFailReason::ChannelError(e.to_string()),
                });
            }
        } else {
            error!(
                "Destination Entity {:?} does not exist anymore",
                outbound.target
            );
            ev_failed.write(MessageSendFailed {
                entity: outbound.target,
                reason: SendFailReason::EntityNotFound,
            });
        }
    }
}

/// 切断済みエンティティを最後にまとめて破棄するシステム。
pub fn despawn_disconnected_system(
    mut commands: Commands,
    mut ev_disconnected: MessageReader<UserDisconnected>,
) {
    for disconnect in ev_disconnected.read() {
        commands.entity(disconnect.entity).despawn();
    }
}
