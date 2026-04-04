use crate::plugins::spatial::{
    SpatialConfig,
    components::{Position2D, Position3D, SpatialZone2D, SpatialZone3D},
    math::{within_radius_2d, within_radius_3d, within_radius_3d_flat},
};
use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Spatial2DPlugin 用
// ============================================================================

/// interest_radius 内の近隣クライアントに位置情報をブロードキャストする（2D）
///
/// # アルゴリズム
/// 1. 全クライアントを収集し、ゾーン → インデックス の HashMap を構築（O(N)）
/// 2. 各クライアント A の隣接ゾーン（最大9）に属するクライアント B だけを距離チェック対象にする
/// 3. interest_radius 以内の B に A の座標を送信
///
/// O(N²) を回避し、O(N × 平均ゾーン密度 × 9) に抑える
///
/// Schedule: FixedUpdate
pub fn broadcast_nearby_2d_system(
    config: Res<SpatialConfig>,
    query: Query<(Entity, &ClientId, &Position2D, &SpatialZone2D)>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    let clients: Vec<_> = query.iter().collect();
    if clients.len() < 2 {
        return;
    }

    // ゾーン → クライアントのインデックス一覧
    let mut zone_map: HashMap<SpatialZone2D, Vec<usize>> = HashMap::new();
    for (i, (_, _, _, zone)) in clients.iter().enumerate() {
        zone_map.entry((*zone).clone()).or_default().push(i);
    }

    let radius = config.interest_radius;

    for (a_entity, a_id, a_pos, a_zone) in &clients {
        let payload = NetworkPayload::Text(format!("pos {} {} {}", a_id.0, a_pos.x, a_pos.y));

        for neighbor_zone in a_zone.neighbors() {
            let Some(indices) = zone_map.get(&neighbor_zone) else {
                continue;
            };
            for &b_idx in indices {
                let (b_entity, _, b_pos, _) = &clients[b_idx];
                if b_entity == a_entity {
                    continue;
                }
                if within_radius_2d(a_pos.x, a_pos.y, b_pos.x, b_pos.y, radius) {
                    ev_send.write(SendMessage {
                        target: *b_entity,
                        payload: payload.clone(),
                    });
                }
            }
        }
    }
}

// ============================================================================
// Spatial3DFlatPlugin 用
// ============================================================================

/// interest_radius 内の近隣クライアントに位置情報をブロードキャストする（3DFlat）
///
/// ゾーンは XZ 平面（SpatialZone2D 流用）なので隣接最大9。
/// 距離チェックも XZ 平面のみで行う（Y軸無視）
///
/// Schedule: FixedUpdate
pub fn broadcast_nearby_3d_flat_system(
    config: Res<SpatialConfig>,
    query: Query<(Entity, &ClientId, &Position3D, &SpatialZone2D)>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    let clients: Vec<_> = query.iter().collect();
    if clients.len() < 2 {
        return;
    }

    let mut zone_map: HashMap<SpatialZone2D, Vec<usize>> = HashMap::new();
    for (i, (_, _, _, zone)) in clients.iter().enumerate() {
        zone_map.entry((*zone).clone()).or_default().push(i);
    }

    let radius = config.interest_radius;

    for (a_entity, a_id, a_pos, a_zone) in &clients {
        let payload = NetworkPayload::Text(format!(
            "pos {} {} {} {}",
            a_id.0, a_pos.x, a_pos.y, a_pos.z
        ));

        for neighbor_zone in a_zone.neighbors() {
            let Some(indices) = zone_map.get(&neighbor_zone) else {
                continue;
            };
            for &b_idx in indices {
                let (b_entity, _, b_pos, _) = &clients[b_idx];
                if b_entity == a_entity {
                    continue;
                }
                // Y軸は無視して XZ のみで判定
                if within_radius_3d_flat(a_pos.x, a_pos.z, b_pos.x, b_pos.z, radius) {
                    ev_send.write(SendMessage {
                        target: *b_entity,
                        payload: payload.clone(),
                    });
                }
            }
        }
    }
}

// ============================================================================
// Spatial3DPlugin 用
// ============================================================================

/// interest_radius 内の近隣クライアントに位置情報をブロードキャストする（完全3D）
///
/// SpatialZone3D の隣接ゾーン（最大27）を列挙してチェック対象を絞る。
/// 距離チェックは XYZ 全軸で行う
///
/// Schedule: FixedUpdate
pub fn broadcast_nearby_3d_system(
    config: Res<SpatialConfig>,
    query: Query<(Entity, &ClientId, &Position3D, &SpatialZone3D)>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    let clients: Vec<_> = query.iter().collect();
    if clients.len() < 2 {
        return;
    }

    let mut zone_map: HashMap<SpatialZone3D, Vec<usize>> = HashMap::new();
    for (i, (_, _, _, zone)) in clients.iter().enumerate() {
        zone_map.entry((*zone).clone()).or_default().push(i);
    }

    let radius = config.interest_radius;

    for (a_entity, a_id, a_pos, a_zone) in &clients {
        let payload = NetworkPayload::Text(format!(
            "pos {} {} {} {}",
            a_id.0, a_pos.x, a_pos.y, a_pos.z
        ));

        for neighbor_zone in a_zone.neighbors() {
            let Some(indices) = zone_map.get(&neighbor_zone) else {
                continue;
            };
            for &b_idx in indices {
                let (b_entity, _, b_pos, _) = &clients[b_idx];
                if b_entity == a_entity {
                    continue;
                }
                if within_radius_3d(a_pos.x, a_pos.y, a_pos.z, b_pos.x, b_pos.y, b_pos.z, radius) {
                    ev_send.write(SendMessage {
                        target: *b_entity,
                        payload: payload.clone(),
                    });
                }
            }
        }
    }
}
