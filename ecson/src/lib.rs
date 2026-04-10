//! # Ecson
//!
//! An ECS-driven, stateful bidirectional server framework for Rust.
//!
//! Ecson combines \[`tokio`\]'s async I/O with \[`bevy_ecs`\]'s data-oriented
//! design, letting you build real-time applications — multiplayer game backends,
//! live collaboration tools, spatial simulations — without ever reaching for
//! `Arc<Mutex<T>>`.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ecson::prelude::*;
//!
//! fn echo_system(
//!     mut ev_recv: MessageReader<MessageReceived>,
//!     mut ev_send: MessageWriter<SendMessage>,
//! ) {
//!     for msg in ev_recv.read() {
//!         ev_send.write(SendMessage {
//!             target: msg.entity,
//!             payload: msg.payload.clone(),
//!         });
//!     }
//! }
//!
//! fn main() {
//!     EcsonApp::new()
//!         .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
//!         .add_systems(Update, echo_system)
//!         .run();
//! }
//! ```
//!
//! ## Feature Overview
//!
//! | Concept | Ecson equivalent |
//! |---|---|
//! | Client connection | \[`Entity`\] |
//! | User state / attributes | \[`Component`\] |
//! | Business logic | System function |
//!
//! ## Crate Layout
//!
//! Start from \[`prelude`\] — it re-exports everything you need.
//! For networking, pick a plugin from \[`EcsonWebSocketPlugin`\] and friends.
//! For built-in features, see the `plugins` module.
//!
//! ⚠️ **Experimental**: APIs may change without notice. Not recommended for production.

/// Built-in Plugin Set.
pub use ecson_ecs::plugins;

// Make ecson_macros' generated code accessible via `::ecson::bevy_ecs::...`
#[doc(hidden)]
pub use bevy_ecs;

// Expose absolute path syntax like `#[ecson::component]` to the crate root
pub use ecson_macros::{component, message, resource};

/// The app prelude.
pub mod prelude {
    pub use ecson_core::app::ShutdownFlag;
    pub use ecson_core::app::{EcsonApp, FixedUpdate, Shutdown, Startup, Update};
    pub use ecson_core::plugin::{Plugin, Plugins, PluginsState};
    pub use ecson_core::server_time_config::ServerTimeConfig;
    pub use ecson_ecs::channels::{NetworkEvent, NetworkPayload};
    pub use ecson_ecs::components::*;
    pub use ecson_ecs::events::*;
    pub use ecson_ecs::resources::*;
    pub use ecson_ecs::systems::*;
    pub use ecson_ecs::types::*;
    pub use ecson_macros::*;
    pub use ecson_network::plugin::*;
}
