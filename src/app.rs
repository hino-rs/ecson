//! サーバーのメインティックループとシステムの登録を管理する `FluxionApp` を定義します。

use std::time::{Duration, Instant};
use bevy_ecs::message::{Message, Messages};
use bevy_ecs::prelude::*;
use crate::plugin::*;
use bevy_ecs::{
    error::ErrorHandler, 
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel}, 
    system::ScheduleSystem, 
    world::World,
    resource::Resource,
};
use crate::ecs::resources::ServerTickRate;


/// メインの実行スケジュールを識別するためのラベル。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MainSchedule;

/// ECSの `World` と実行 `Schedule` を管理するコアアプリケーション構造体。
/// 
/// プラグインやシステムの登録、およびメイン実行ループの駆動を担います。
#[must_use]
pub struct FluxionApp {
    /// 全てのエンティティ、コンポーネント、リソースを保持するECSワールド。
    pub world: World,
    /// システムが登録され、実行されるメインスケジュール。
    pub schedule: Schedule,
    /// デフォルトのエラーハンドラ。
    default_error_handler: Option<ErrorHandler>,
}

impl Default for FluxionApp {
    /// 空の `World` と `MainSchedule` を持つデフォルトインスタンスを作成します。
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
    /// 新しい `FluxionApp` インスタンスを作成します。
    pub fn new() -> FluxionApp {
        FluxionApp::default()
    }

    /// サーバーのメインループを開始します。
    /// 
    /// `ServerTickRate` に基づいた固定レート（デフォルト60Hz）でスケジュールを実行します。
    pub fn run(&mut self) {
        println!("FluxionApp🚀");

        let tick_rate = self.world
            .get_resource::<ServerTickRate>()
            .map(|r| r.0)
            .unwrap_or(60.0);
        let target_duration = Duration::from_secs_f64(1.0 / tick_rate);

        // サーバーのメインループ
        loop {
            let frame_start = Instant::now();

            // スケジュールに登録されたすべてのシステムを実行
            self.schedule.run(&mut self.world);

            let elapsed = frame_start.elapsed();

            // 目標時間よりも早く処理が終わった場合は、残りの時間だけスリープする
            if elapsed < target_duration {
                std::thread::sleep(target_duration - elapsed);
            } else {
                eprintln!("[Warning] Server is lagging! Tick took {elapsed:?}");
            }
        }
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

            self.add_systems(
                MainSchedule,
                |mut msgs: ResMut<Messages<M>>| msgs.update()
            );
        }
        self
    }

    /// プラグインの現在の状態を取得します。
    /// 
    /// （※現在は `Ready` 固定ですが、将来的なステート管理のために用意されています）
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
    /// 
    /// ※注意: 現在の実装では引数 `_schedule` が使用されず、常に `self.schedule` に追加されます。
    /// 複数のスケジュール（Startup, Updateなど）を分ける場合は実装の修正が必要です。
    pub fn add_systems<M>(
        &mut self,
        _schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedule.add_systems(systems);
        self
    }
}