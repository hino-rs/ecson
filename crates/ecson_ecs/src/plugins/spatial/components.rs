use bevy_ecs::prelude::*;

// ============================================================================
// Position コンポーネント
// ============================================================================

/// 2D 空間上のクライアント位置（XY 平面）
/// Spatial2DPlugin で使用
#[derive(Component, Clone, Debug, Default)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

/// 3D 空間上のクライアント位置（XYZ）
/// Spatial3DFlatPlugin / Spatial3DPlugin で使用
#[derive(Component, Clone, Debug, Default)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ============================================================================
// SpatialZone コンポーネント
// ============================================================================

/// 2D ゾーン（XY 平面グリッド）
/// Spatial2DPlugin と Spatial3DFlatPlugin（XZ 平面）で共用
///
/// Spatial3DFlatPlugin では x=chunk_x, y=chunk_z として扱う
// HashMap のキーに使うため Clone が必要
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpatialZone2D {
    pub zone_x: i32,
    pub zone_y: i32,
}

/// 3D ゾーン（XYZ ボリューメトリックグリッド）
/// Spatial3DPlugin 専用
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpatialZone3D {
    pub zone_x: i32,
    pub zone_y: i32,
    pub zone_z: i32,
}

// ============================================================================
// ゾーン計算ユーティリティ
// ============================================================================

impl SpatialZone2D {
    /// 座標からゾーンを算出する
    #[inline]
    pub fn from_pos(x: f32, y: f32, zone_size: f32) -> Self {
        Self {
            zone_x: (x / zone_size).floor() as i32,
            zone_y: (y / zone_size).floor() as i32,
        }
    }

    /// 隣接ゾーン（自身含む最大9セル）を返す
    #[inline]
    pub fn neighbors(&self) -> [SpatialZone2D; 9] {
        let mut i = 0;
        let mut result = std::array::from_fn(|_| SpatialZone2D::default());
        for dy in -1..=1 {
            for dx in -1..=1 {
                result[i] = SpatialZone2D {
                    zone_x: self.zone_x + dx,
                    zone_y: self.zone_y + dy,
                };
                i += 1;
            }
        }
        result
    }
}

impl SpatialZone3D {
    /// 座標からゾーンを算出する
    #[inline]
    pub fn from_pos(x: f32, y: f32, z: f32, zone_size: f32) -> Self {
        Self {
            zone_x: (x / zone_size).floor() as i32,
            zone_y: (y / zone_size).floor() as i32,
            zone_z: (z / zone_size).floor() as i32,
        }
    }

    /// 隣接ゾーン（自身含む最大27セル）を返す
    #[inline]
    pub fn neighbors(&self) -> [SpatialZone3D; 27] {
        let mut i = 0;
        let mut result = std::array::from_fn(|_| SpatialZone3D::default());
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    result[i] = SpatialZone3D {
                        zone_x: self.zone_x + dx,
                        zone_y: self.zone_y + dy,
                        zone_z: self.zone_z + dz,
                    };
                    i += 1;
                }
            }
        }
        result
    }
}
