pub use bevy_ecs;
pub use ecson_core;
pub use ecson_ecs;
pub use ecson_network;

pub use ecson_ecs::plugins;

pub mod prelude {
    pub use bevy_ecs::event::Event;
    pub use bevy_ecs::message::{MessageReader, MessageWriter, Messages};
    pub use bevy_ecs::prelude::*;

    pub use ecson_core::app::{EcsonApp, FixedUpdate, Startup, Update};
    pub use ecson_core::plugin::Plugin;
    pub use ecson_core::server_time_config::ServerTimeConfig;

    pub use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
    pub use ecson_ecs::components::*;
    pub use ecson_ecs::events::*;
    pub use ecson_ecs::resources::*;
    pub use ecson_ecs::systems::*;

    pub use ecson_network::plugin::*;
}
