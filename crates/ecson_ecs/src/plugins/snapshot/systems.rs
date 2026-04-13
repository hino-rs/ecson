// use super::{
//     SnapshotConfig, SnapshotFrame, SnapshotMode, SnapshotRegistry, SnapshotSentEvent,
//     SnapshotState, SnapshotSubscriber, SubscribeSnapshotEvent, UnsubscribeSnapshotEvent,
// };
// use crate::prelude::*;
// use bevy_ecs::prelude::*;

// // ============================================================================
// // 購読管理
// // ============================================================================

// /// `SubscribeSnapshotEvent` を受け取り、対象エンティティに `SnapshotSubscriber` を付与する。
// ///
// /// 新規購読者にはフルスナップショットを即時送信する。
// pub fn handle_subscribe_system(
//     mut commands: Commands,
//     mut ev_sub: MessageReader<SubscribeSnapshotEvent>,
// ) {
//     for ev in ev_sub.read() {
//         commands.entity(ev.target.entity()).insert(SnapshotSubscriber);
//     }
// }

// /// `UnsubscribeSnapshotEvent` を受け取り、対象エンティティから `SnapshotSubscriber` を除去する。
// pub fn handle_unsubscribe_system(
//     mut commands: Commands,
//     mut ev_unsub: MessageReader<UnsubscribeSnapshotEvent>,
// ) {
//     for ev in ev_unsub.read() {
//         commands.entity(ev.target.entity()).remove::<SnapshotSubscriber>();
//     }
// }

// // ============================================================================
// // 収集
// // ============================================================================

// /// `Snapshotable` エンティティの現在状態を収集し `SnapshotState::pending` に格納するシステム。
// ///
// /// # 動作フロー
// /// 1. インターバル未経過なら即リターン
// /// 2. `SnapshotRegistry` の各コレクターを実行してエンティティスナップショットを収集
// /// 3. `SnapshotMode::Full` → 全エンティティを `upserted` に積む
// /// 4. `SnapshotMode::Delta` → 前回との差分（追加・変更・削除）のみを積む
// /// 5. 差分なしなら `pending` を `None` のままにして broadcast をスキップさせる
// pub fn collect_snapshot_system(world: &mut World,) {
//     // インターバルチェック
//     let (interval_secs, mode) = {
//         let config = world.resource::<SnapshotConfig>();
//         (config.interval_secs, config.mode)
//     };
//     {
//         let state = world.resource::<SnapshotState>();
//         if state.last_sent.elapsed().as_secs_f32() < interval_secs {
//             return;
//         }
//     }

//     // Registryを一時的にWorldから取り出す
//     let Some(registry) = world.remove_resource::<SnapshotRegistry>() else {
//         return;
//     };

//     let last_snapshot = world.resource::<SnapshotState>().last_snapshot.clone();

//     // 収納
//     let frame = match mode {
//         SnapshotMode::Full  => collect_full(&registry, world),
//         SnapshotMode::Delta => collect_delta(&registry, world, &last_snapshot),
//     };

//     // RegistryをWorldに戻す
//     world.insert_resource(registry);

//     // pendingに収納
//     let mut state = world.resource_mut::<SnapshotState>();
//     state.pending = if frame.is_empty() { None } else { Some(frame) };
//     state.last_sent = std::time::Instant::now();
// }

// /// Full モード用の収集ロジック。全 Snapshotable エンティティを返す。
// fn collect_full(registry: &SnapshotRegistry, world: &World) -> SnapshotFrame {
//     todo!()
// }

// /// Delta モード用の収集ロジック。前回スナップショットとの差分のみを返す。
// fn collect_delta(
//     registry: &SnapshotRegistry,
//     world: &World,
//     last: &std::collections::HashMap<u32, std::collections::HashMap<&'static str, String>>,
// ) -> SnapshotFrame {
//     todo!()
// }

// // ============================================================================
// // 送信
// // ============================================================================

// /// `pending` に格納されたスナップショットを全 `SnapshotSubscriber` へ送信するシステム。
// ///
// /// # ペイロード形式（JSON）
// /// ```json
// /// // Full
// /// {"type":"snapshot","seq":0,"mode":"full","upserted":[...],"removed":[]}
// ///
// /// // Delta
// /// {"type":"snapshot","seq":1,"mode":"delta","upserted":[...],"removed":[42]}
// /// ```
// pub fn broadcast_snapshot_system(
//     mut state: ResMut<SnapshotState>,
//     config: Res<SnapshotConfig>,
//     subscribers: Query<Entity, With<SnapshotSubscriber>>,
//     mut ev_send: MessageWriter<SendMessage>,
//     mut ev_sent: MessageWriter<SnapshotSentEvent>,
// ) {
//     todo!()
// }

// /// `SnapshotFrame` を JSON 文字列にシリアライズする。
// fn serialize_frame(seq: u64, mode: SnapshotMode, frame: &SnapshotFrame) -> String {
//     todo!()
// }
