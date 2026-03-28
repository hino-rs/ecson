pub use bevy_ecs;

pub mod app;
pub mod ecs;
pub mod network;
pub mod plugin;
pub mod server;

pub mod prelude {
    pub use crate::bevy_ecs::prelude::*;

    pub use crate::bevy_ecs::event::Event;
    pub use crate::bevy_ecs::message::{MessageReader, MessageWriter, Messages};

    pub use crate::app::{FluxionApp, MainSchedule};
    pub use crate::ecs::components::*;
    pub use crate::ecs::events::MessageReceived;
    pub use crate::ecs::systems::*;
    pub use crate::network::channels::NetworkEvent;
    pub use crate::ecs::events::SendMessage;


    pub use crate::plugin::FluxionWebSocketPlugin;
    pub use crate::plugin::FluxionWebTransportPlugin;
}
