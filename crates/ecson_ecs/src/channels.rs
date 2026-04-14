//! Defines the message and event types exchanged between the network layer
//! (async Tokio tasks) and the ECS layer (synchronous game loop).

use tokio::sync::mpsc;

/// Payload for data sent or received over the network.
/// Supports both text (e.g. JSON) and binary data.
#[derive(Debug, Clone)]
pub enum NetworkPayload {
    /// Text data.
    Text(String),
    /// Binary data.
    Binary(Vec<u8>),
}

/// Events sent from the network layer to the ECS layer.
pub enum NetworkEvent {
    /// Indicates that a new client has established a connection.
    Connected {
        id: u64,
        sender: mpsc::Sender<NetworkPayload>,
    },

    /// Indicates that a message was received from a client.
    Message { id: u64, payload: NetworkPayload },

    /// Indicates that a client connection was closed.
    Disconnected { id: u64 },
}
