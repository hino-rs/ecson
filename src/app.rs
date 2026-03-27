//! Defines the `FluxionApp` struct, which allows users to register systems
//! and run the main tick loop for the server.

use std::time::{Duration, Instant};
use bevy_ecs::{message::{Message, Messages}};
use crate::plugin::*;
use bevy_ecs::{
    error::ErrorHandler, 
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel}, 
    system::ScheduleSystem, 
    world::World,
    resource::Resource,
};
use crate::ecs::resources::ServerTickRate;

/// A label used to identify the main execution schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MainSchedule;

/// The core application structure that manages the ECS `World` and execution `Schedule`.
/// 
/// It is responsible for registering plugins and systems, and driving the main execution loop.
#[must_use]
pub struct FluxionApp {
    /// The ECS world containing all entities, components, and resources.
    pub world: World,
    /// The main schedule where systems are registered and executed.
    pub schedule: Schedule,
    default_error_handler: Option<ErrorHandler>,
}

impl Default for FluxionApp {
    /// Creates a default instance of `FluxionApp` with an empty `World` and `MainSchedule`.
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(ServerTickRate::default());

        FluxionApp {
            world,
            schedule: Schedule::new(MainSchedule),
            default_error_handler: None,
        }
    }
}

impl FluxionApp {
    pub fn new() -> FluxionApp {
        FluxionApp::default()
    }

    pub fn run(&mut self) {
        println!("FluxionApp🚀");

        let tick_rate = self.world
            .get_resource::<ServerTickRate>()
            .map(|r| r.0)
            .unwrap_or(60.0);
        let target_duration = Duration::from_secs_f64(1.0 / tick_rate);

        // Server main loop
        loop {
            // ループの開始時刻を記録
            let frame_start = Instant::now();

            // Run all systems registered in the schedule
            self.schedule.run(&mut self.world);

            // 実行にかかった時間を計測
            let elapsed = frame_start.elapsed();

            // 目標時間よりも早く処理が終わった場合は、残りの時間だけスリープする
            if elapsed < target_duration {
                std::thread::sleep(target_duration - elapsed);
            } else {
                eprintln!("[Warning] Server is lagging! Tick took {elapsed:?}");
            }
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn add_event<M: Message>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Messages<M>>() {
            self.world.insert_resource(Messages::<M>::default());
        }
        self.world.insert_resource(Messages::<M>::default());
        self
    }

    #[inline]
    pub fn plugins_state(&mut self) -> PluginsState {
        PluginsState::Ready
    }

    pub fn add_plugins<P: Plugins>(&mut self, plugins: P) -> &mut Self {
        plugins.add_to_app(self);
        self
    }

    pub fn add_systems<M>(
        &mut self,
        _schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedule.add_systems(systems);
        self
    }
}