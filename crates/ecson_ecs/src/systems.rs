//! ECS systems for receiving network events and dispatching outbound messages to clients.

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

/// Resource wrapper holding the receiver for network events sent from the Tokio side.
#[derive(Resource)]
pub struct NetworkReceiver(pub mpsc::Receiver<NetworkEvent>);

/// System that polls network layer events and updates ECS state accordingly.
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
                info!("ECS: new connection {id} -> Entity {entity:?}");
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
                info!("ECS: {} disconnected", id);

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

/// System that processes outbound `SendMessage` events issued within ECS and forwards them to the network layer.
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

/// System that despawns disconnected entities at the end of the frame.
pub fn despawn_disconnected_system(
    mut commands: Commands,
    mut ev_disconnected: MessageReader<UserDisconnected>,
) {
    for disconnect in ev_disconnected.read() {
        commands.entity(disconnect.entity).despawn();
    }
}
