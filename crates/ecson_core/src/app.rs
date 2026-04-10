use crate::plugin::*;
use crate::server_time_config::ServerTimeConfig;
use bevy_ecs::error::DefaultErrorHandler;
use bevy_ecs::message::{Message, Messages};
use bevy_ecs::prelude::*;
use bevy_ecs::{
    error::ErrorHandler,
    resource::Resource,
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    system::ScheduleSystem,
    world::World,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, warn};

// ============================================================================
// Define Schedule Label
// ============================================================================

/// A schedule that runs **only once** when the server starts.
///
/// This is ideal for setup processes that should only occur once across
/// the entire application, such as starting network listeners or inserting
/// initial resources.
///
/// # Examples
/// ```rust,ignore
/// app.add_systems(Startup, setup_system);
/// ```
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

/// A schedule that runs **every frame** in the main loop.
///
/// This is suited for input-related systems that you want to process
/// as frequently as possible, such as network reception and event polling.
/// The execution frequency depends on the `update_sleep` configuration.
///
/// # Examples
/// ```rust,ignore
/// app.add_systems(Update, input_system);
/// ```
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// A schedule that runs at **fixed intervals**.
///
/// This is suited for systems that require temporal stability, such as
/// game logic, physics calculations, and state management. The execution
/// interval is determined by `ServerTimeConfig::tick_rate`.
///
/// If the frame rate drops, this schedule may run up to `max_ticks_per_frame`
/// times per frame. If the accumulated time exceeds this limit, the
/// accumulator is reset (time skip).
///
/// # Examples
/// ```rust,ignore
/// app.add_systems(FixedUpdate, physics_system);
/// ```
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixedUpdate;

/// A schedule that runs **only once** just before the server shuts down.
///
/// This is ideal for cleanup processes upon termination, such as sending
/// disconnect notifications to connected clients or flushing persistent data.
///
/// # Examples
/// ```rust,ignore
/// app.add_systems(Shutdown, cleanup_system);
/// ```
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shutdown;

// ============================================================================
// Tokio runtime resources
// ============================================================================

/// An ECS resource for the Tokio runtime shared across the entire application.
///
/// Rather than generating an independent `Runtime` for each network plugin,
/// you can prevent duplicate Tokio thread pools and improve resource efficiency
/// by cloning an `Arc` from this shared resource and calling `spawn`.
///
/// # Usage
///
/// Within a plugin, you can obtain a reference to spawn asynchronous tasks:
///
/// ```rust,ignore
/// let rt = app.world.get_resource::<TokioRuntime>().unwrap().clone();
/// rt.spawn(async move {
///     // Asynchronous processing
/// });
/// ```
#[derive(Resource, Clone)]
pub struct TokioRuntime(pub Arc<tokio::runtime::Runtime>);

impl TokioRuntime {
    /// Spawns an asynchronous task on the underlying `Runtime`.
    ///
    /// This is a convenience method that clones the internal `Arc` to `spawn`.
    /// If the returned [`JoinHandle`] is dropped, the task is detached and
    /// continues to run in the background.
    ///
    /// [`JoinHandle`]: tokio::task::JoinHandle
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

/// A termination flag for the main loop.
///
/// This wraps an `Arc<AtomicBool>`, allowing it to be safely shared and
/// cloned across multiple threads or asynchronous tasks.
///
/// When a signal handler (`Ctrl+C` / `SIGTERM`) is triggered, the internal
/// flag is set to `true`, and the main loop gracefully exits on the next frame.
///
/// # Examples
///
/// To request a shutdown from an ECS system, use [`EcsonApp::request_shutdown`].
///
/// ```rust,ignore
/// app.add_systems(Update, EcsonApp::request_shutdown);
/// ```
#[derive(Resource, Clone, Default)]
pub struct ShutdownFlag(pub Arc<AtomicBool>);

impl ShutdownFlag {
    /// Creates the flag in a `false` state.
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Requests a shutdown.
    ///
    /// Sets the flag to `true`. The main loop will detect this flag on the
    /// next frame, execute the `Shutdown` schedule, and terminate.
    /// It is safe to call this concurrently from multiple threads.
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Returns whether a shutdown has been requested.
    ///
    /// If `true`, the main loop will terminate after completing the current frame.
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

// ============================================================================
// Application Core
// ============================================================================

/// The entry point of the Ecson application.
///
/// Integrates the ECS [`World`] and the scheduler, managing the entire lifecycle
/// of the server. It centralizes APIs required to build an application, such as
/// registering plugins, inserting resources, and adding systems.
///
/// # Lifecycle
///
/// ```text
/// new() → add_plugins() / add_systems() / insert_resource()
///       → run()
///           ├─ Startup      (Runs once)
///           ├─ Update       (Runs every frame)
///           ├─ FixedUpdate  (Runs at fixed intervals)
///           └─ Shutdown     (Runs once just before exiting)
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use ecson::prelude::*;
///
/// EcsonApp::new()
///     .add_plugins(EcsonWebSocketPlugin::default())
///     .add_systems(Startup, setup)
///     .add_systems(FixedUpdate, game_logic)
///     .run();
///
/// fn setup() { /* Initialization */ }
/// fn game_logic() { /* Game Logic */ }
/// ```
#[must_use]
pub struct EcsonApp {
    /// The ECS world that manages all entities, components, and resources.
    world: World,
    /// A mapping of schedule labels to the actual schedules.
    /// If a non-existent label is specified during `add_systems`, it is inserted automatically.
    pub(crate) schedules: Schedules,
    /// A list of registered plugins. Their `cleanup` method is called during shutdown.
    pub(crate) plugins: Vec<Box<dyn Plugin>>,
    /// The current state of the plugins. Transitions in order: `Adding` → `Ready` → `Finished` → `Cleaned`.
    pub(crate) plugins_state: PluginsState,
}

impl Default for EcsonApp {
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(ServerTimeConfig::default());
        world.insert_resource(ShutdownFlag::new());
        // Create a single Tokio runtime here and register it with the World.
        // All network plugins will then share this resource.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime.");
        world.insert_resource(TokioRuntime(Arc::new(rt)));
        let mut schedules = Schedules::new();
        schedules.insert(Schedule::new(Startup));
        schedules.insert(Schedule::new(Update));
        schedules.insert(Schedule::new(FixedUpdate));
        schedules.insert(Schedule::new(Shutdown));
        EcsonApp {
            world,
            schedules,
            plugins: Vec::new(),
            plugins_state: PluginsState::Adding,
        }
    }
}

impl EcsonApp {
    /// Creates a new `EcsonApp` instance.
    ///
    /// The following resources and schedules are initialized by default:
    ///
    /// **Resources**
    /// - [`ServerTimeConfig`] — Timing configurations like tick rate.
    /// - [`ShutdownFlag`] — Termination flag for the main loop.
    /// - [`TokioRuntime`] — Shared Tokio multi-thread runtime.
    ///
    /// **Schedules**
    /// - [`Startup`] / [`Update`] / [`FixedUpdate`] / [`Shutdown`]
    ///
    /// # Panics
    ///
    /// Panics if the Tokio multi-thread runtime fails to initialize
    /// (usually due to failing to spawn OS threads).
    pub fn new() -> EcsonApp {
        EcsonApp::default()
    }

    /// Starts the server lifecycle.
    ///
    /// Executes processes in the following order and blocks until shutdown is complete:
    ///
    /// 1. Runs the `Startup` schedule once.
    /// 2. Registers `Ctrl+C` / `SIGTERM` signal handlers as asynchronous tasks.
    /// 3. Starts the main loop:
    ///    - Every frame: Runs `Update`.
    ///    - Fixed intervals: Runs `FixedUpdate` (up to `max_ticks_per_frame` times).
    ///    - End of frame: Sleeps for `update_sleep` duration.
    /// 4. Calls the `cleanup` method for all registered plugins after detecting the shutdown flag.
    /// 5. Runs the `Shutdown` schedule once before terminating.
    pub fn run(&mut self) {
        info!("EcsonApp Started🚀");
        self.plugins_state = PluginsState::Ready;

        // =========================================================
        // Startup
        // =========================================================
        if let Some(startup_schedule) = self.schedules.get_mut(Startup) {
            startup_schedule.run(&mut self.world);
        }

        let flag = self.world.get_resource::<ShutdownFlag>().unwrap().clone();
        let flag1 = flag.clone();
        let rt = self.world.get_resource::<TokioRuntime>().unwrap().clone();

        rt.spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl_c");
            info!("Shutdown signal received.");
            flag1.request();
        });

        #[cfg(unix)]
        {
            let flag2 = flag.clone();
            rt.spawn(async move {
                let mut sig =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .unwrap();
                sig.recv().await;
                info!("SIGTERM received");
                flag2.request();
            });
        }

        let config = self
            .world
            .get_resource::<ServerTimeConfig>()
            .cloned()
            .unwrap_or_default();
        let fixed_timestep = Duration::from_secs_f64(1.0 / config.tick_rate);
        let max_ticks_per_frame = config.max_ticks_per_frame;
        let update_sleep = Duration::from_micros((config.update_sleep * 1000.0) as u64);
        let mut previous_time = Instant::now();
        let mut accumulator = Duration::ZERO;

        // =========================================================
        // Main Loop
        // =========================================================
        while !flag.is_requested() {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(previous_time);
            previous_time = current_time;
            accumulator += delta_time;

            // -----------------------------------------------------
            // Update
            // -----------------------------------------------------
            if let Some(update_schedule) = self.schedules.get_mut(Update) {
                update_schedule.run(&mut self.world);
            }

            // -----------------------------------------------------
            // FixedUpdate
            // Process accumulated data in batches
            // -----------------------------------------------------
            let mut ticks = 0;
            while accumulator >= fixed_timestep {
                if let Some(fixed_update) = self.schedules.get_mut(FixedUpdate) {
                    fixed_update.run(&mut self.world);
                }
                accumulator -= fixed_timestep;
                ticks += 1;
                if ticks >= max_ticks_per_frame {
                    if config.warn_on_lag {
                        warn!("[Warning] Server is severely lagging! Skipping fixed frames.");
                    }
                    accumulator = Duration::ZERO;
                    break;
                }
            }

            // -----------------------------------------------------
            // Sleep to reduce CPU load
            // -----------------------------------------------------
            std::thread::sleep(update_sleep);
        }

        self.plugins_state = PluginsState::Finished;
        info!("Cleaning up plugins...");
        for plugin in &self.plugins {
            plugin.cleanup(&mut self.world);
        }
        self.plugins_state = PluginsState::Cleaned;
        info!("Running Shutdown schedule...");
        if let Some(shutdown_schedule) = self.schedules.get_mut(Shutdown) {
            shutdown_schedule.run(&mut self.world);
        }
        info!("EcsonApp stopped gracefully.");
    }

    /// Sets the ECS error handler.
    ///
    /// Used when you want to customize how errors that occur during system execution
    /// are handled. The default error handler is `bevy_ecs`'s [`DefaultErrorHandler`].
    pub fn set_error_handler(&mut self, handler: ErrorHandler) -> &mut Self {
        self.world.insert_resource(DefaultErrorHandler(handler));
        self
    }

    /// Inserts a resource into the `EcsonApp`'s [`World`].
    ///
    /// If a resource of the same type already exists, it will be overwritten.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Resource)]
    /// struct MyConfig { value: u32 }
    ///
    /// app.insert_resource(MyConfig { value: 42 });
    /// ```
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Returns whether the specified resource exists in the [`World`].
    pub fn contains_resource<R: Resource>(&mut self) -> bool {
        self.world.contains_resource::<R>()
    }

    /// Returns a reference to the specified resource.
    ///
    /// Returns `None` if the resource does not exist.
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.world.get_resource::<R>()
    }

    /// Registers a resource for the event type `M` and its clearing system.
    ///
    /// - Inserts the [`Messages<M>`] resource into the [`World`].
    /// - Adds `msgs.update()` to the `Update` schedule to clear old messages at the end of every frame.
    ///
    /// It is safe (idempotent) to call this multiple times for the same type.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Message, Clone)]
    /// struct PlayerMoved { id: u64, x: f32, y: f32 }
    ///
    /// app.add_event::<PlayerMoved>();
    /// ```
    pub fn add_event<M: Message>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Messages<M>>() {
            self.world.insert_resource(Messages::<M>::default());
            self.add_systems(Update, |mut msgs: ResMut<Messages<M>>| msgs.update());
        }
        self
    }

    /// Returns the current state of the plugins.
    ///
    /// The state transitions in the following order: `Adding` → `Ready` → `Finished` → `Cleaned`.
    #[inline]
    pub fn plugins_state(&self) -> PluginsState {
        self.plugins_state
    }

    /// Adds plugins to the application.
    ///
    /// You can register a single plugin implementing the `Plugins` trait, or
    /// multiple plugins simultaneously using a tuple.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// app.add_plugins(EcsonWebSocketPlugin::default());
    /// // Register multiple plugins simultaneously
    /// app.add_plugins((HeartbeatPlugin::default(), RateLimitPlugin::default()));
    /// ```
    pub fn add_plugins<P: Plugins>(&mut self, plugins: P) -> &mut Self {
        plugins.add_to_app(self);
        self
    }

    /// Adds systems to the specified schedule.
    ///
    /// If a non-existent schedule label is provided, it will be automatically created.
    /// Systems can be registered individually or as tuples as long as they implement `IntoScheduleConfigs`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// app.add_systems(Update, (recv_messages, process_input).chain());
    /// app.add_systems(FixedUpdate, physics_system);
    /// ```
    pub fn add_systems<M, L>(
        &mut self,
        schedule_label: L,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self
    where
        L: ScheduleLabel + Clone,
    {
        if !self.schedules.contains(schedule_label.clone()) {
            self.schedules.insert(Schedule::new(schedule_label.clone()));
        }
        self.schedules
            .get_mut(schedule_label)
            .unwrap()
            .add_systems(systems);
        self
    }

    /// A shutdown request helper that can be called as an ECS system.
    ///
    /// It receives the [`ShutdownFlag`] as a system parameter and requests a shutdown.
    /// Pass this directly into `add_systems`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Conditional shutdown
    /// app.add_systems(Update, EcsonApp::request_shutdown.run_if(should_stop));
    ///
    /// fn should_stop() -> bool { /* termination condition */ false }
    /// ```
    pub fn request_shutdown(flag: Res<ShutdownFlag>) {
        flag.request();
    }

    /// Runs the `Startup` schedule exactly once.
    ///
    /// This is primarily used in testing when you only want to run the initialization logic.
    /// Since `run()` executes this automatically, it is not usually necessary.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut app = EcsonApp::new();
    /// app.add_systems(Startup, setup);
    /// app.startup(); // setup() is executed once
    /// ```
    pub fn startup(&mut self) {
        if let Some(startup) = self.schedules.get_mut(Startup) {
            startup.run(&mut self.world);
        }
    }

    /// Runs `Update` and `FixedUpdate` exactly once.
    ///
    /// This is mainly used in tests to manually advance the logic by a single frame.
    /// Since the elapsed time for `FixedUpdate` is not considered, use [`tick_n`]
    /// if you need to simulate accumulated timesteps.
    ///
    /// [`tick_n`]: EcsonApp::tick_n
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// app.startup();
    /// app.tick_once(); // Advance by 1 frame
    /// ```
    pub fn tick_once(&mut self) {
        if let Some(update_schedule) = self.schedules.get_mut(Update) {
            update_schedule.run(&mut self.world);
        }
        if let Some(fixed_update) = self.schedules.get_mut(FixedUpdate) {
            fixed_update.run(&mut self.world);
        }
    }

    /// Repeats the main loop of `Update` and `FixedUpdate` `n` times.
    ///
    /// By measuring the actual elapsed time to accumulate timesteps, the number
    /// of times `FixedUpdate` is executed per frame will vary. This is mainly
    /// used for running tests or reproducing scenarios.
    ///
    /// # Arguments
    ///
    /// * `n` — The number of frames to run.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// app.startup();
    /// app.tick_n(60); // Advance by 60 frames
    /// ```
    pub fn tick_n(&mut self, n: u128) {
        let config = self
            .world
            .get_resource::<ServerTimeConfig>()
            .cloned()
            .unwrap_or_default();
        let fixed_timestep = Duration::from_secs_f64(1.0 / config.tick_rate);
        let max_ticks_per_frame = config.max_ticks_per_frame;
        let update_sleep = Duration::from_micros((config.update_sleep * 1000.0) as u64);
        let mut previous_time = Instant::now();
        let mut accumulator = Duration::ZERO;

        for _ in 0..n {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(previous_time);
            previous_time = current_time;
            accumulator += delta_time;

            if let Some(update_schedule) = self.schedules.get_mut(Update) {
                update_schedule.run(&mut self.world);
            }

            let mut ticks = 0;
            while accumulator >= fixed_timestep {
                if let Some(fixed_update) = self.schedules.get_mut(FixedUpdate) {
                    fixed_update.run(&mut self.world);
                }
                accumulator -= fixed_timestep;
                ticks += 1;
                if ticks >= max_ticks_per_frame {
                    if config.warn_on_lag {
                        warn!("[Warning] Server is severely lagging! Skipping fixed frames.");
                    }
                    accumulator = Duration::ZERO;
                    break;
                }
            }

            std::thread::sleep(update_sleep);
        }
    }
}
