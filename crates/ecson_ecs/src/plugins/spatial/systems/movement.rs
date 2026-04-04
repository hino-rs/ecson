use crate::plugins::spatial::{
    SpatialConfig,
    components::{Position2D, Position3D, SpatialZone2D, SpatialZone3D},
    events::{ClientMovedEvent, ClientZoneChangedEvent, MovePayload},
};
use crate::prelude::*;
use bevy_ecs::prelude::*;

// ============================================================================
// セットアップシステム
// ============================================================================

/// 新規接続クライアントに Position2D / SpatialZone2D を付与する
///
/// ClientId は持っているが Position2D を持っていないエンティティが対象。
pub fn setup_spatial_2d_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<Position2D>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((Position2D::default(), SpatialZone2D::default()));
    }
}

/// 新規接続クライアントに Position3D / SpatialZone2D(XZ用) を付与する
pub fn setup_spatial_3d_flat_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<Position3D>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((Position3D::default(), SpatialZone2D::default()));
    }
}

/// 新規接続クライアントに Position3D / SpatialZone3D を付与する
pub fn setup_spatial_3d_system(
    mut commands: Commands,
    query: Query<Entity, (With<ClientId>, Without<Position3D>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((Position3D::default(), SpatialZone3D::default()));
    }
}

// ============================================================================
// Spatial2DPlugin 用
// ============================================================================

/// ClientMovedEvent(Move2D) を処理して Position2D / SpatialZone2D を更新する
///
/// - Position2D を新座標で上書き
/// - ゾーンが変わった場合のみ SpatialZone2D を更新し、ClientZoneChangedEvent を発行
///
/// Schedule: FixedUpdate
pub fn handle_move_2d_system(
    mut ev_moved: MessageReader<ClientMovedEvent>,
    mut ev_zone_changed: MessageWriter<ClientZoneChangedEvent>,
    config: Res<SpatialConfig>,
    mut query: Query<(&mut Position2D, &mut SpatialZone2D)>,
) {
    for ev in ev_moved.read() {
        let MovePayload::Move2D { x, y } = ev.payload else {
            continue;
        };
        let Ok((mut pos, mut zone)) = query.get_mut(ev.entity) else {
            continue;
        };

        pos.x = x;
        pos.y = y;

        let new_zone = SpatialZone2D::from_pos(x, y, config.zone_size);
        if new_zone != *zone {
            *zone = new_zone;
            ev_zone_changed.write(ClientZoneChangedEvent {
                entity: ev.entity,
                client_id: ev.client_id,
            });
        }
    }
}

// ============================================================================
// Spatial3DFlatPlugin 用
// ============================================================================

/// ClientMovedEvent(Move3D) を処理して Position3D / SpatialZone2D(XZ) を更新する
///
/// - Position3D を新座標で上書き（Y軸も保持）
/// - ゾーン計算は XZ 平面のみ（SpatialZone2D 流用）
/// - ゾーンが変わった場合のみ SpatialZone2D を更新し、ClientZoneChangedEvent を発行
///
/// Schedule: FixedUpdate
pub fn handle_move_3d_flat_system(
    mut ev_moved: MessageReader<ClientMovedEvent>,
    mut ev_zone_changed: MessageWriter<ClientZoneChangedEvent>,
    config: Res<SpatialConfig>,
    mut query: Query<(&mut Position3D, &mut SpatialZone2D)>,
) {
    for ev in ev_moved.read() {
        let MovePayload::Move3D { x, y, z } = ev.payload else {
            continue;
        };
        let Ok((mut pos, mut zone)) = query.get_mut(ev.entity) else {
            continue;
        };

        pos.x = x;
        pos.y = y;
        pos.z = z;

        // XZ 平面でゾーン計算（zone_y を Z チャンクとして使う）
        let new_zone = SpatialZone2D::from_pos(x, z, config.zone_size);
        if new_zone != *zone {
            *zone = new_zone;
            ev_zone_changed.write(ClientZoneChangedEvent {
                entity: ev.entity,
                client_id: ev.client_id,
            });
        }
    }
}

// ============================================================================
// Spatial3DPlugin 用
// ============================================================================

/// ClientMovedEvent(Move3D) を処理して Position3D / SpatialZone3D を更新する
///
/// - Position3D を新座標で上書き
/// - ゾーンが変わった場合のみ SpatialZone3D を更新し、ClientZoneChangedEvent を発行
///
/// Schedule: FixedUpdate
pub fn handle_move_3d_system(
    mut ev_moved: MessageReader<ClientMovedEvent>,
    mut ev_zone_changed: MessageWriter<ClientZoneChangedEvent>,
    config: Res<SpatialConfig>,
    mut query: Query<(&mut Position3D, &mut SpatialZone3D)>,
) {
    for ev in ev_moved.read() {
        let MovePayload::Move3D { x, y, z } = ev.payload else {
            continue;
        };
        let Ok((mut pos, mut zone)) = query.get_mut(ev.entity) else {
            continue;
        };

        pos.x = x;
        pos.y = y;
        pos.z = z;

        let new_zone = SpatialZone3D::from_pos(x, y, z, config.zone_size);
        if new_zone != *zone {
            *zone = new_zone;
            ev_zone_changed.write(ClientZoneChangedEvent {
                entity: ev.entity,
                client_id: ev.client_id,
            });
        }
    }
}
