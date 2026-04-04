pub mod channels;
pub mod components;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use ecson_core::app::{EcsonApp, FixedUpdate, Startup, Update};
    pub use ecson_core::plugin::Plugin;
    pub use bevy_ecs::prelude::*;
    pub use bevy_ecs::event::Event;
    pub use bevy_ecs::message::{MessageReader, MessageWriter, Messages};
    pub use crate::channels::{NetworkEvent, NetworkPayload};
    pub use crate::components::*;
    pub use crate::events::*;
    pub use crate::resources::*;
    pub use crate::systems::*;
}
