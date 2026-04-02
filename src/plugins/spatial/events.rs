use bevy_ecs::prelude::*;
use bevy_ecs::message::Message;

// ============================================================================
// 移動ペイロード
// ============================================================================

/// 移動の次元を区別
#[derive(Clone, Debug)]
pub enum MovePayload {
    /// "/move x y" → 2D / 3DFlat 共用
    Move2D { x: f32, y: f32 },
    /// "/move x y z"
    Move3D { x: f32, y: f32, z: f32 },
}

// ============================================================================
// 内部イベント（プラグイン → システム間）
// ============================================================================

/// クライアントが移動したときに発火する内部イベント
#[derive(Message, Clone, Debug)]
pub struct ClientMovedEvent {
    pub entity: Entity,
    pub client_id: u64,
    pub payload: MovePayload,
}

// ============================================================================
// フックイベント（ユーザー向け）
// ============================================================================

/// クライアントが別のゾーンへ移動した時に発火する
/// ゾーン遷移に応じたゲームロジック（マップロード等）を実装する際に利用する
#[derive(Message, Clone, Debug)]
pub struct ClientZoneChangedEvent {
    pub entity: Entity,
    pub client_id: u64,
}

// ============================================================================
// パースユーティリティ
// ============================================================================

/// "/move x y [z]" 形式のテキストを MovePayload にパースする
///
/// - "/move 1.0 2.0"       → Move2D
/// - "/move 1.0 2.0 3.0"   → Move3D
/// - それ以外               → None
pub fn parse_move_command(text: &str) -> Option<MovePayload> {
    let mut parts = text.split_whitespace();

    if parts.next()? != "/move" {
        return None;
    }

    let x: f32 = parts.next()?.parse().ok()?;
    let y: f32 = parts.next()?.parse().ok()?;

    match parts.next() {
        Some(z_str) => {
            let z: f32 = z_str.parse().ok()?;
            Some(MovePayload::Move3D { x, y, z })
        }
        None => Some(MovePayload::Move2D { x, y })
    }
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_2d() {
        let p = parse_move_command("/move 1.5 2.5").unwrap();
        assert!(matches!(p, MovePayload::Move2D { x, y } if x == 1.5 && y == 2.5));
    }

    #[test]
    fn test_parse_3d() {
        let p = parse_move_command("/move 1.0 2.0 3.0").unwrap();
        assert!(matches!(p, MovePayload::Move3D { x, y, z } if x == 1.0 && y == 2.0 && z == 3.0));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_move_command("/chat hello").is_none());
        assert!(parse_move_command("/move abc").is_none());
        assert!(parse_move_command("").is_none());
    }
}