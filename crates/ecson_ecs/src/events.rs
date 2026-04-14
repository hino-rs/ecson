//! Defines events (Messages) exchanged between clients and systems.

use crate::channels::NetworkPayload;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::Message;

/// Reason for a message send failure.
#[derive(Debug)]
pub enum SendFailReason {
    /// `try_send` returned an error (channel full, already disconnected, etc.).
    ChannelError(String),
    /// The target entity no longer exists in the World.
    EntityNotFound,
}

// =====================================================
// Message send / receive / failure
// =====================================================

/// Event for sending a message from the server to a specific client.
#[derive(Message)]
pub struct SendMessage {
    pub target: Entity,
    pub payload: NetworkPayload,
}

/// Event fired when a message is received from a client.
#[derive(Message)]
pub struct MessageReceived {
    pub entity: Entity,
    pub client_id: u64,
    pub payload: NetworkPayload,
}

/// Event fired when sending a message fails.
#[derive(Message)]
pub struct MessageSendFailed {
    pub entity: Entity,
    pub reason: SendFailReason,
}

// =====================================================
// Client connect / disconnect
// =====================================================

/// Event fired when a client connects.
#[derive(Message)]
pub struct UserConnected {
    pub entity: Entity,
    pub client_id: u64,
}

/// Event fired when a client disconnects.
#[derive(Message)]
pub struct UserDisconnected {
    pub entity: Entity,
    pub client_id: u64,
}
