pub mod app;
pub mod plugin;
pub mod server_time_config;

pub mod prelude {
    pub use crate::app::{EcsonApp, FixedUpdate, Startup, Update};
    pub use crate::plugin::{Plugin, Plugins, PluginsState};
    pub use crate::server_time_config::ServerTimeConfig;
}
