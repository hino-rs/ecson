use bevy_ecs::prelude::*;
use crate::prelude::*;
mod systems;
use systems::*;

// ============================================================================
// コンポーネント
// ============================================================================

/// 2D 空間上のクライアント位置
#[derive(Component, Clone, Debug, Default)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

/// 3D 空間上のクライアント位置
#[derive(Component, Clone, Debug, Default)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// クライアントが属するゾーン/チャンク（空間分割最適化用）
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpatialZone {
    pub zone_x: i32,
    pub zone_y: i32,
}

// ============================================================================
// リソース
// ============================================================================

/// 空間プラグインの設定
#[derive(Resource)]
pub struct SpatialConfig {
    /// 近接検索の最大距離
    pub interest_radius: f32,
    /// ゾーンのセルサイズ
    pub zone_size: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0,
        }
    }
}

// ============================================================================
// イベント
// ============================================================================

/// クライアントが移動したときに発火
#[derive(Message)]
pub struct ClientMovedEvent {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
    pub z: Option<f32>,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct SpatialPlugin {
    pub interest_radius: f32,
    pub zone_size: f32,
}

impl Default for SpatialPlugin {
    fn default() -> Self {
        Self {
            interest_radius: 100.0,
            zone_size: 50.0,
        }
    }
}

impl SpatialPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interest_radius(mut self, radius: f32) -> Self {
        self.interest_radius = radius;
        self
    }

    pub fn zone_size(mut self, size: f32) -> Self {
        self.zone_size = size;
        self
    }
}

impl Plugin for SpatialPlugin {
    fn build(self, app: &mut EcsonApp) {
        app.world.insert_resource(SpatialConfig {
            interest_radius: self.interest_radius,
            zone_size: self.zone_size,
        });

        app.add_event::<ClientMovedEvent>();

        app.add_systems(Update, parse_move_messages_system);
        app.add_systems(
            FixedUpdate,
            (
                handle_client_move_system,
                broadcast_nearby_positions_system,
            ),
        );
    }
}
