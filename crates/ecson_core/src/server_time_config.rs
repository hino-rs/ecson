use bevy_ecs::resource::Resource;

/// Configuration for server tick timing.
#[derive(Resource, Clone)]
pub struct ServerTimeConfig {
    /// Interval between Update ticks (seconds).
    pub update_sleep: f64,
    /// Target tick rate for FixedUpdate (Hz).
    pub tick_rate: f64,
    /// Maximum number of FixedUpdate ticks allowed per frame to catch up.
    pub max_ticks_per_frame: u32,
    /// Whether to emit a warning log when ticks are dropped.
    pub warn_on_lag: bool,
}

impl Default for ServerTimeConfig {
    fn default() -> Self {
        Self {
            update_sleep: 0.01,
            tick_rate: 60.0,
            max_ticks_per_frame: 5,
            warn_on_lag: false,
        }
    }
}
