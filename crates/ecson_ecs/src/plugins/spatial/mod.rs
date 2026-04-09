use crate::prelude::*;
use bevy_ecs::prelude::*;

pub mod components;
pub mod events;
pub mod math;
mod systems;

pub use components::*;
pub use events::*;

// ============================================================================
// 共通リソース
// ============================================================================

/// 空間プラグイン共通の設定
///
/// # ゾーンサイズの制約
/// `zone_size >= interest_radius / 2` を守ること。
/// これより小さいと隣接ゾーンのみのチェックでは AOI 漏れが発生する。
#[derive(Resource)]
pub struct SpatialConfig {
    /// AOI（近接通知）の最大距離
    pub interest_radius: f32,
    /// ゾーンのセルサイズ
    pub zone_size: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0, // >= interest_radius / 2 を満たす
        }
    }
}

// ============================================================================
// Spatial2DPlugin  （XY 平面）
// ============================================================================

/// 2D 空間の AOI・位置同期プラグイン
///
/// # 使用コンポーネント
/// - [`Position2D`]
/// - [`SpatialZone2D`]
pub struct Spatial2DPlugin {
    interest_radius: f32,
    zone_size: f32,
}

impl Default for Spatial2DPlugin {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0,
        }
    }
}

impl Spatial2DPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interest_radius(mut self, r: f32) -> Self {
        self.interest_radius = r;
        self
    }

    pub fn zone_size(mut self, s: f32) -> Self {
        self.zone_size = s;
        self
    }
}

impl Plugin for Spatial2DPlugin {
    fn build(&self, app: &mut EcsonApp) {
        app.insert_resource(SpatialConfig {
            interest_radius: self.interest_radius,
            zone_size: self.zone_size,
        });

        app.add_event::<ClientMovedEvent>();
        app.add_event::<ClientZoneChangedEvent>();

        app.add_systems(
            Update,
            (
                systems::setup_spatial_2d_system,
                systems::parse_move_messages_system,
                systems::handle_move_2d_system,
                systems::broadcast_nearby_2d_system,
            )
                .chain(),
        );
    }
}

// ============================================================================
// Spatial3DFlatPlugin  （XZ 平面 AOI、Y軸は位置のみ保持）
// ============================================================================

/// 地上系 3D ゲーム向けの AOI・位置同期プラグイン
///
/// `Position3D` で Y 軸の座標を保持しつつ、
/// AOI のゾーン計算は XZ 平面（`SpatialZone2D` 流用）で行う。
/// 隣接ゾーン数が最大 9 で済むため、完全 3D より低コスト。
///
/// # ユースケース
/// - 地上を移動する RPG / MOBA
/// - マインクラフト的なワールド（高さ方向は AOI 不要）
///
/// # 使用コンポーネント
/// - [`Position3D`]
/// - [`SpatialZone2D`]（XZ として利用、zone_y = chunk_z）
pub struct Spatial3DFlatPlugin {
    interest_radius: f32,
    zone_size: f32,
}

impl Default for Spatial3DFlatPlugin {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0,
        }
    }
}

impl Spatial3DFlatPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interest_radius(mut self, r: f32) -> Self {
        self.interest_radius = r;
        self
    }

    pub fn zone_size(mut self, s: f32) -> Self {
        self.zone_size = s;
        self
    }
}

impl Plugin for Spatial3DFlatPlugin {
    fn build(&self, app: &mut EcsonApp) {
        app.insert_resource(SpatialConfig {
            interest_radius: self.interest_radius,
            zone_size: self.zone_size,
        });

        app.add_event::<ClientMovedEvent>();
        app.add_event::<ClientZoneChangedEvent>();

        app.add_systems(
            Update,
            (
                systems::setup_spatial_3d_flat_system,
                systems::parse_move_messages_system,
                systems::handle_move_3d_flat_system,
                systems::broadcast_nearby_3d_flat_system,
            )
                .chain(),
        );
    }
}

// ============================================================================
// Spatial3DPlugin  （XYZ ボリューメトリック）
// ============================================================================

/// 完全 3D 空間の AOI・位置同期プラグイン
///
/// XYZ すべての軸でゾーン分割を行う。隣接ゾーンが最大 27 になるため、
/// `Spatial3DFlatPlugin` より AOI チェックのコストが高い。
/// 宇宙ゲーム・飛行シム・多層ダンジョン等で使用する。
///
/// # 使用コンポーネント
/// - [`Position3D`]
/// - [`SpatialZone3D`]
pub struct Spatial3DPlugin {
    interest_radius: f32,
    zone_size: f32,
}

impl Default for Spatial3DPlugin {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0,
        }
    }
}

impl Spatial3DPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interest_radius(mut self, r: f32) -> Self {
        self.interest_radius = r;
        self
    }

    pub fn zone_size(mut self, s: f32) -> Self {
        self.zone_size = s;
        self
    }
}

impl Plugin for Spatial3DPlugin {
    fn build(&self, app: &mut EcsonApp) {
        app.insert_resource(SpatialConfig {
            interest_radius: self.interest_radius,
            zone_size: self.zone_size,
        });

        app.add_event::<ClientMovedEvent>();
        app.add_event::<ClientZoneChangedEvent>();

        app.add_systems(
            Update,
            (
                systems::setup_spatial_3d_system,
                systems::parse_move_messages_system,
                systems::handle_move_3d_system,
                systems::broadcast_nearby_3d_system,
            )
                .chain(),
        );
    }
}
