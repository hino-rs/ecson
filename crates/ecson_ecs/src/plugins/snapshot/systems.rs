use super::{SnapshotConfig, SnapshotSentEvent, SnapshotState, SnapshotSubscriber, Snapshotable};
use crate::prelude::*;
use bevy_ecs::prelude::*;

/// Snapshotable エンティティの現在状態を収集してシリアライズするシステム
///
/// # 動作フロー
/// 1. `interval_secs` が経過していなければ即リターン
/// 2. `Snapshotable` エンティティを収集して JSON にシリアライズ
/// 3. `delta_only: true` かつ前回と差分なし → `pending` を空にしてスキップ
/// 4. 差分あり（またはフル送信モード）→ `pending` にバイト列を格納
///
/// broadcast_snapshot_system が `pending` を読んで送信する。
///
/// # スケジュール: FixedUpdate（collect → broadcast の順で実行）
pub fn collect_snapshot_system(
    config: Res<SnapshotConfig>,
    mut state: ResMut<SnapshotState>,
    query: Query<(Entity, &bevy_ecs::name::Name), With<Snapshotable>>,
) {
    // インターバルチェック
    if state.last_sent.elapsed().as_secs_f32() < config.interval_secs {
        state.pending.clear(); // broadcast に「今回は送らない」を伝える
        return;
    }

    // Snapshotable エンティティを簡易 JSON でシリアライズ
    // 例: [{"id":1,"name":"player_a"},{"id":2,"name":"player_b"}]
    let entries: Vec<String> = query
        .iter()
        .map(|(entity, name)| format!(r#"{{"id":{},"name":"{}"}}"#, entity.index(), name.as_str()))
        .collect();
    let bytes = format!("[{}]", entries.join(",")).into_bytes();

    // delta_only: 前回と同じなら送信をスキップ
    if config.delta_only && bytes == state.last_snapshot {
        state.pending.clear();
        return;
    }

    state.pending = bytes;
}

/// 収集したスナップショットを SnapshotSubscriber クライアントへ送信するシステム
///
/// `collect_snapshot_system` が `pending` を空にした場合は何もしない。
///
/// # スケジュール: FixedUpdate（collect の直後に実行）
pub fn broadcast_snapshot_system(
    config: Res<SnapshotConfig>,
    mut state: ResMut<SnapshotState>,
    subscribers: Query<Entity, With<SnapshotSubscriber>>,
    mut ev_send: MessageWriter<SendMessage>,
    mut ev_sent: MessageWriter<SnapshotSentEvent>,
) {
    // pending が空 = collect がスキップ → 何もしない
    if state.pending.is_empty() {
        return;
    }

    let text = String::from_utf8_lossy(&state.pending).to_string();
    let byte_size = state.pending.len();

    for target in subscribers.iter() {
        ev_send.write(SendMessage {
            target,
            payload: NetworkPayload::Text(text.clone()),
        });
    }

    ev_sent.write(SnapshotSentEvent {
        sequence: state.sequence,
        delta: config.delta_only,
        byte_size,
    });

    // ステートを更新
    state.last_snapshot = std::mem::take(&mut state.pending);
    state.sequence += 1;
    state.last_sent = std::time::Instant::now();
}
