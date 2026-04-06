//! サーバーのメインティックループとシステムの登録を管理する `EcsonApp` を定義します。

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
// スケジュールラベルの定義
// ============================================================================

/// サーバー起動時に1回だけ実行されるスケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

/// 毎フレーム(可能な限り高速に)実行されるスケジュール (ネットワーク受信など)
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// 固定時間ごとに実行されるスケジュール (ゲームロジックや状態動機など)
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixedUpdate;

/// サーバー終了直前に1回だけ実行されるスケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shutdown;

// ============================================================================
// Tokio ランタイムリソース
// ============================================================================

/// アプリケーション全体で共有される Tokio ランタイムを保持する ECS リソース。
///
/// ネットワークプラグインはプラグインごとに Runtime を生成するのではなく、
/// このリソースから `Arc` のクローンを取得して `spawn` する。
/// これにより Tokio のスレッドプールが一元管理され、リソースの重複生成を防ぐ。
///
/// # 使用例（プラグイン内）
/// ```rust, ignore
/// let rt = app.world.get_resource::<TokioRuntime>().unwrap().clone();
/// rt.spawn(async move { ... });
/// ```
#[derive(Resource, Clone)]
pub struct TokioRuntime(pub Arc<tokio::runtime::Runtime>);

impl TokioRuntime {
    /// 内包する `Runtime` 上に非同期タスクをスポーンする。
    /// `Arc` を複数箇所でクローンして使うための便利メソッド。
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

/// ループ終了フラグ
#[derive(Resource, Clone, Default)]
pub struct ShutdownFlag(pub Arc<AtomicBool>);

impl ShutdownFlag {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

// ============================================================================
// アプリケーションコア
// ============================================================================
/// ECSの `World` と実行 `Schedule` を管理するコアアプリケーション構造体。
///
/// プラグインやシステムの登録、およびメイン実行ループの駆動を担います。
#[must_use]
pub struct EcsonApp {
    /// 全てのエンティティ、コンポーネント、リソースを保持するECSワールド。
    pub world: World,
    /// システムが登録され、実行されるスケジュール方式。
    pub schedules: Schedules,
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl Default for EcsonApp {
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(ServerTimeConfig::default());
        world.insert_resource(ShutdownFlag::new());

        // Tokio ランタイムをここで1つだけ生成し、World に登録する。
        // 以降、全ネットワークプラグインはこのリソースを共有する。
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Tokio runtime の初期化に失敗しました");
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
        }
    }
}

impl EcsonApp {
    /// 新しい `EcsonApp` インスタンスを作成します。
    pub fn new() -> EcsonApp {
        EcsonApp::default()
    }

    /// サーバーのメインループを開始します。
    pub fn run(&mut self) {
        info!("EcsonApp Started🚀");

        // =========================================================
        // Startup スケジュールの実行（サーバー起動時に1回だけ）
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
                .expect("failed to listen for ctrl_c");
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

        // サーバーのコンフィグを取得
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
        // メインループ（Update は毎フレーム高速回転）
        // =========================================================
        while !flag.is_requested() {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(previous_time);
            previous_time = current_time;

            accumulator += delta_time;

            // -----------------------------------------------------
            // Update スケジュールの実行（毎フレーム・受信ポーリング）
            // sleep なしで可能な限り高速に回す。
            // -----------------------------------------------------
            if let Some(update_schedule) = self.schedules.get_mut(Update) {
                update_schedule.run(&mut self.world);
            }

            // -----------------------------------------------------
            // FixedUpdate スケジュールの実行（固定時間ごとに実行）
            // accumulator が溜まった分だけまとめて処理する。
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
            // CPU負荷軽減のためのスリープ
            // fixed_timestep ではなく固定の短い間隔でスリープする。
            // Update は高速に回しつつ、CPU を無駄に食い潰さない。
            // 例: 1ms スリープ → Update は最大 ~1000回/秒 まで回る。
            // -----------------------------------------------------
            std::thread::sleep(update_sleep);
        }
        info!("Cleaning up plugins...");
        for plugin in &self.plugins {
            plugin.cleanup(&mut self.world);
        }
        info!("Running Shutdown schedule...");
        if let Some(shutdown_schedule) = self.schedules.get_mut(Shutdown) {
            shutdown_schedule.run(&mut self.world);
        }
        info!("EcsonApp stopped gracefully.");
    }

    /// エラーハンドラを設定します。
    pub fn set_error_handler(&mut self, handler: ErrorHandler) -> &mut Self {
        self.world.insert_resource(DefaultErrorHandler(handler));
        self
    }

    /// `World` にリソースを追加します。
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// イベント（Message）を処理するためのリソースと更新システムを登録します。
    pub fn add_event<M: Message>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Messages<M>>() {
            self.world.insert_resource(Messages::<M>::default());

            self.add_systems(Update, |mut msgs: ResMut<Messages<M>>| msgs.update());
        }
        self
    }

    /// プラグインの現在の状態を取得します。
    #[inline]
    pub fn plugins_state(&mut self) -> PluginsState {
        PluginsState::Ready
    }

    /// プラグインをアプリケーションに追加します。
    pub fn add_plugins<P: Plugins>(&mut self, plugins: P) -> &mut Self {
        plugins.add_to_app(self);
        self
    }

    /// システムをスケジュールに追加します。
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

    /// システム内から呼べるシャットダウン要求ヘルパー
    pub fn request_shutdown(flag: Res<ShutdownFlag>) {
        flag.request();
    }

    /// Startupを1回実行します
    pub fn startup(&mut self) {
        if let Some(startup) = self.schedules.get_mut(Startup) {
            startup.run(&mut self.world);
        }
    }

    /// UpdateとFixedUpdateを1回実行します
    pub fn tick_once(&mut self) {
        if let Some(update_schedule) = self.schedules.get_mut(Update) {
            update_schedule.run(&mut self.world);
        }
        if let Some(fixed_update) = self.schedules.get_mut(FixedUpdate) {
            fixed_update.run(&mut self.world);
        }
    }

    /// UpdateとFixedUpdateをn回実行します
    pub fn tick_n(&mut self, n: u128) {
        // サーバーのコンフィグを取得
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
        // メインループ
        // =========================================================
        for _ in 0..n {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(previous_time);
            previous_time = current_time;

            accumulator += delta_time;

            // -----------------------------------------------------
            // Update スケジュールの実行
            // -----------------------------------------------------
            if let Some(update_schedule) = self.schedules.get_mut(Update) {
                update_schedule.run(&mut self.world);
            }

            // -----------------------------------------------------
            // FixedUpdate スケジュールの実行
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
            // CPU負荷軽減のためのスリープ
            // -----------------------------------------------------
            std::thread::sleep(update_sleep);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_has_default_config() {
        let app = EcsonApp::new();
        let config = app.world.get_resource::<ServerTimeConfig>().unwrap();
        assert_eq!(config.tick_rate, 60.0); // デフォルト値
    }

    #[test]
    fn tokio_runtime_can_spawn_task() {
        // Runtime 上でタスクをスポーンし、結果が取得できることを確認する
        let app = EcsonApp::new();
        let rt = app.world.get_resource::<TokioRuntime>().unwrap().clone();
        let handle = rt.spawn(async { 42u32 });
        let result = rt.0.block_on(handle).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn add_systems_creates_schedule_if_missing() {
        let mut app = EcsonApp::new();
        #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
        struct MySchedule;
        app.add_systems(MySchedule, || {});
        assert!(app.schedules.contains(MySchedule));
    }

    #[test]
    fn insert_resource_is_accessible() {
        #[derive(Resource)]
        struct Marker(u32);
        let mut app = EcsonApp::new();
        app.insert_resource(Marker(42));
        assert_eq!(app.world.get_resource::<Marker>().unwrap().0, 42);
    }
}
