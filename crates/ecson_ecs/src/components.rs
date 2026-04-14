//! ECS components representing the attributes and state of connected clients.

use crate::channels::NetworkPayload;
use bevy_ecs::prelude::*;
use tokio::sync::mpsc;

/// Component holding the channel used to send data to the network layer.
#[derive(Component)]
pub struct ClientSender(pub mpsc::Sender<NetworkPayload>);

/// Component that uniquely identifies a client by its network ID.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

/// Component representing the room name the client is currently in.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Room(pub String);

/// Component holding the display name (nickname) of a client.
#[derive(Component)]
pub struct Username(pub String);
