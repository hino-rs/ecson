use bevy_ecs::resource::Resource;

/// サーバーの回転に関するコンフィグ
#[derive(Resource, Clone)]
pub struct ServerTimeConfig {
    /// Updateの間隔 (ms)
    pub update_sleep: f64,
    /// FixedUpdateの目標Tickレート(Hz)
    pub tick_rate: f64,
    /// 1フレーム内で後れを取り戻すために実行できるFixedUpdateの最大回数
    pub max_ticks_per_frame: u32,
    /// 初理落ち時に警告ログを出すかどうか
    pub warn_on_lag: bool,
}

impl Default for ServerTimeConfig {
    fn default() -> Self {
        Self {
            update_sleep: 1.0,
            tick_rate: 60.0,
            max_ticks_per_frame: 5,
            warn_on_lag: false,
        }
    }
}
