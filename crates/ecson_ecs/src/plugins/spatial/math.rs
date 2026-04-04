//! 空間計算のユーティリティ関数群
//!
//! システムから直接参照する純粋関数のみを置く。
//! ヒープアロケーションなし、ECS 依存なし。

// ============================================================================
// 2D 距離
// ============================================================================

/// 2点間のユークリッド距離（2D）
#[inline]
pub fn dist_2d(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    dist_sq_2d(ax, ay, bx, by).sqrt()
}

/// 2点間の距離の二乗（2D）
/// sqrt を省けるため、半径との比較には squared 版を使うと速い
#[inline]
pub fn dist_sq_2d(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy
}

/// 2点が interest_radius 以内か判定（2D）
/// 内部で dist_sq_2d を使い sqrt を回避する
#[inline]
pub fn within_radius_2d(ax: f32, ay: f32, bx: f32, by: f32, radius: f32) -> bool {
    dist_sq_2d(ax, ay, bx, by) <= radius * radius
}

// ============================================================================
// 3D 距離（XYZ）
// ============================================================================

/// 2点間のユークリッド距離（3D）
#[inline]
pub fn dist_3d(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> f32 {
    dist_sq_3d(ax, ay, az, bx, by, bz).sqrt()
}

/// 2点間の距離の二乗（3D）
#[inline]
pub fn dist_sq_3d(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    let dz = az - bz;
    dx * dx + dy * dy + dz * dz
}

/// 2点が interest_radius 以内か判定（3D / XYZ）
#[inline]
pub fn within_radius_3d(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32, radius: f32) -> bool {
    dist_sq_3d(ax, ay, az, bx, by, bz) <= radius * radius
}

// ============================================================================
// 3DFlat 距離（XZ 平面のみ、Y軸無視）
// ============================================================================

/// 2点間の距離（XZ 平面のみ）
#[inline]
pub fn dist_3d_flat(ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    dist_sq_3d_flat(ax, az, bx, bz).sqrt()
}

/// 2点間の距離の二乗（XZ 平面のみ）
#[inline]
pub fn dist_sq_3d_flat(ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    let dx = ax - bx;
    let dz = az - bz;
    dx * dx + dz * dz
}

/// 2点が interest_radius 以内か判定（XZ 平面のみ）
#[inline]
pub fn within_radius_3d_flat(ax: f32, az: f32, bx: f32, bz: f32, radius: f32) -> bool {
    dist_sq_3d_flat(ax, az, bx, bz) <= radius * radius
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 2D ---

    #[test]
    fn test_dist_2d() {
        // 3-4-5 三角形
        assert_eq!(dist_2d(0.0, 0.0, 3.0, 4.0), 5.0);
    }

    #[test]
    fn test_dist_sq_2d() {
        assert_eq!(dist_sq_2d(0.0, 0.0, 3.0, 4.0), 25.0);
    }

    #[test]
    fn test_within_radius_2d_inside() {
        assert!(within_radius_2d(0.0, 0.0, 3.0, 4.0, 5.0));
    }

    #[test]
    fn test_within_radius_2d_on_edge() {
        // ちょうど境界上は含む
        assert!(within_radius_2d(0.0, 0.0, 3.0, 4.0, 5.0));
    }

    #[test]
    fn test_within_radius_2d_outside() {
        assert!(!within_radius_2d(0.0, 0.0, 3.0, 4.0, 4.9));
    }

    // --- 3D ---

    #[test]
    fn test_dist_3d() {
        // (0,0,0) → (1,2,2) = 3.0
        assert_eq!(dist_3d(0.0, 0.0, 0.0, 1.0, 2.0, 2.0), 3.0);
    }

    #[test]
    fn test_dist_sq_3d() {
        assert_eq!(dist_sq_3d(0.0, 0.0, 0.0, 1.0, 2.0, 2.0), 9.0);
    }

    #[test]
    fn test_within_radius_3d() {
        assert!(within_radius_3d(0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 3.0));
        assert!(!within_radius_3d(0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.9));
    }

    // --- 3DFlat ---

    #[test]
    fn test_dist_3d_flat() {
        // XZ だけで 3-4-5
        assert_eq!(dist_3d_flat(0.0, 0.0, 3.0, 4.0), 5.0);
    }

    #[test]
    fn test_within_radius_3d_flat_ignores_y() {
        // Y 軸が大きく離れていても XZ が近ければ true になることを
        // 呼び出し側（broadcast）が Y を渡さないことで保証する
        assert!(within_radius_3d_flat(0.0, 0.0, 3.0, 4.0, 5.0));
        assert!(!within_radius_3d_flat(0.0, 0.0, 3.0, 4.0, 4.9));
    }
}
