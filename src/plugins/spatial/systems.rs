use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{ClientMovedEvent, Position2D, SpatialConfig, SpatialZone};

/// 受信メッセージから移動コマンドを解析するシステム
pub fn parse_move_messages_system(
    _ev_received: MessageReader<MessageReceived>,
    _ev_moved: MessageWriter<ClientMovedEvent>,
) {
    todo!("'/move x y [z]' 等のメッセージを解析し、ClientMovedEvent を発行")
}

/// ClientMovedEvent を処理して Position/Zone コンポーネントを更新するシステム
pub fn handle_client_move_system(
    _ev_moved: MessageReader<ClientMovedEvent>,
    _config: Res<SpatialConfig>,
    _query: Query<(&mut Position2D, &mut SpatialZone)>,
) {
    todo!("Position2D / Position3D を更新し、ゾーン境界を超えたら SpatialZone も更新")
}

/// 近隣クライアントに位置情報をブロードキャストするシステム
pub fn broadcast_nearby_positions_system(
    _config: Res<SpatialConfig>,
    _query: Query<(Entity, &Position2D, &SpatialZone)>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("interest_radius 内のクライアントにのみ位置情報を送信（AOI: Area of Interest）")
}
