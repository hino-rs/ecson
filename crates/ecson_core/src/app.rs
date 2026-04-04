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
}

impl Default for EcsonApp {
    /// 空の `World` と `MainSchedule` を持つデフォルトインスタンスを作成します。
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(ServerTimeConfig::default());

        let mut schedules = Schedules::new();
        schedules.insert(Schedule::new(Startup));
        schedules.insert(Schedule::new(Update));
        schedules.insert(Schedule::new(FixedUpdate));

        EcsonApp { world, schedules }
    }
}

impl EcsonApp {
    /// 新しい `EcsonApp` インスタンスを作成します。
    pub fn new() -> EcsonApp {
        EcsonApp::default()
    }

    /// サーバーのメインループを開始します。
    pub fn run(&mut self) {
        println!("EcsonApp Started🚀");

        // =========================================================
        // Startup スケジュールの実行（サーバー起動時に1回だけ）
        // =========================================================
        if let Some(startup_schedule) = self.schedules.get_mut(Startup) {
            startup_schedule.run(&mut self.world);
        }

        // サーバーのコンフィグを取得
        let config = self
            .world
            .get_resource::<ServerTimeConfig>()
            .cloned()
            .unwrap_or_default();

        let fixed_timestep = Duration::from_secs_f64(1.0 / config.tick_rate);
        let max_ticks_per_frame = config.max_ticks_per_frame;

        let mut previous_time = Instant::now();
        let mut accumulator = Duration::ZERO;

        // =========================================================
        // メインループ
        // =========================================================
        loop {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(previous_time);
            previous_time = current_time;

            accumulator += delta_time;

            // -----------------------------------------------------
            // Update スケジュールの実行（毎フレーム実行）
            // -----------------------------------------------------
            if let Some(update_schedule) = self.schedules.get_mut(Update) {
                update_schedule.run(&mut self.world);
            }

            // -----------------------------------------------------
            // FixedUpdate スケジュールの実行（固定時間ごとに実行）
            // -----------------------------------------------------
            let mut frames_processed = 0;
            while accumulator >= fixed_timestep {
                if let Some(fixed_update) = self.schedules.get_mut(FixedUpdate) {
                    fixed_update.run(&mut self.world);
                }
                accumulator -= fixed_timestep;
                frames_processed += 1;

                // 無限ループ対策
                if frames_processed >= max_ticks_per_frame {
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
            let elapsed_since_current = current_time.elapsed();
            if elapsed_since_current < fixed_timestep {
                std::thread::sleep(fixed_timestep - elapsed_since_current);
            }
        }
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
}
