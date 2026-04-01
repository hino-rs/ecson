use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{SnapshotConfig, SnapshotState, SnapshotSentEvent, Snapshotable, SnapshotSubscriber};

/// Snapshotable エンティティの現在状態を収集してシリアライズするシステム
pub fn collect_snapshot_system(
    _config: Res<SnapshotConfig>,
    _snapshot_state: ResMut<SnapshotState>,
    _query: Query<(Entity, &bevy_ecs::name::Name), With<Snapshotable>>,
) {
    todo!(
        "Snapshotable コンポーネントを持つ全エンティティの状態を収集し、\
        delta_only が true の場合は last_snapshot との差分のみ SnapshotState に保存"
    )
}

/// 収集したスナップショットを SnapshotSubscriber クライアントへ送信するシステム
pub fn broadcast_snapshot_system(
    _config: Res<SnapshotConfig>,
    _snapshot_state: Res<SnapshotState>,
    _subscribers: Query<Entity, With<SnapshotSubscriber>>,
    _ev_send: MessageWriter<SendMessage>,
    _ev_sent: MessageWriter<SnapshotSentEvent>,
) {
    todo!(
        "SnapshotState の内容を SnapshotSubscriber 全員に送信し、\
        SnapshotSentEvent を発行して sequence をインクリメント"
    )
}
