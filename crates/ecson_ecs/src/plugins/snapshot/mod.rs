// use crate::prelude::*;
// use bevy_ecs::prelude::*;
// use std::collections::HashMap;

// mod systems;
// use systems::*;

// // ============================================================================
// // トレイト
// // ============================================================================

// /// スナップショットに含めるコンポーネントに実装するトレイト。
// ///
// /// # 例
// /// ```rust,ignore
// /// #[component]
// /// struct Position { x: f32, y: f32 }
// ///
// /// impl SnapshotData for Position {
// ///     fn component_name() -> &'static str { "position" }
// ///     fn to_snapshot_json(&self) -> String {
// ///         format!(r#"{{"x":{},"y":{}}}"#, self.x, self.y)
// ///     }
// /// }
// /// ```
// pub trait SnapshotData: Component {
//     /// ペイロード内でのコンポーネントキー名
//     fn component_name() -> &'static str;

//     /// このコンポーネントの状態をJSON文字列にシリアライズする
//     fn to_snapshot_json(&self) -> String;
// }

// // ============================================================================
// // 型
// // ============================================================================

// /// スナップショットの送信モード
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum SnapshotMode {
//     /// 全エンティティの全コンポーネントを送信
//     Full,
//     /// 前回から変化のあったエンティティ・コンポーネントのみ送信
//     Delta,
// }

// /// 1エンティティ分のスナップショットデータ
// #[derive(Debug, Clone)]
// pub struct EntitySnapshot {
//     /// `Entity::index()`
//     pub entity_id: u32,
//     /// コンポーネント名 → JSON文字列
//     pub components: HashMap<&'static str, String>,
// }

// /// 1フレーム分のスナップショット (collect → broadcast間のバッファ)
// #[derive(Debug, Clone)]
// pub struct SnapshotFrame {
//     /// 追加・フル送信対象のエンティティ
//     pub upserted: Vec<EntitySnapshot>,
//     /// 削除されたエンティティのID
//     pub removed: Vec<u32>,
// }

// impl SnapshotFrame {
//     pub fn is_empty(&self) -> bool {
//         self.upserted.is_empty() && self.removed.is_empty()
//     }
// }

// // ============================================================================
// // リソース
// // ============================================================================

// /// スナップショットプラグインの設定
// #[derive(Resource)]
// pub struct SnapshotConfig {
//     /// スナップショット送信間隔（秒）
//     pub interval_secs: f32,
//     /// 差分のみ送信するか（false の場合フル送信）
//     pub mode: SnapshotMode,
// }

// impl Default for SnapshotConfig {
//     fn default() -> Self {
//         Self {
//             interval_secs: 0.1, // 10Hz
//             mode: SnapshotMode::Delta,
//         }
//     }
// }

// /// 登録された型ごとのコレクター関数を保持するレジストリ
// ///
// /// `SnapshotPlugin::register::<T>()` によって登録される。
// /// 各コレクターは World から該当コンポーネントを持つエンティティを収集する。
// #[derive(Resource, Default)]
// pub struct SnapshotRegistry {
//     /// (component_name, collector)
//     /// collector: &World → Vec<(entity_id, json)>
//     pub(crate) collectors: Vec<(
//         &'static str,
//         Box<dyn Fn(&mut World) -> Vec<(u32, String)> + Send + Sync>,
//     )>,
// }

// impl SnapshotRegistry {
//     pub fn new() -> Self {
//         Self::default()
//     }
// }

// /// スナップショットの実行状態を保持するリソース
// #[derive(Resource)]
// pub struct SnapshotState {
//     /// 送信済みシーケンス番号
//     pub sequence: u64,
//     /// 前回送信時の各エンティティのスナップショット（差分検知用）
//     /// entity_id → component_name → json
//     pub last_snapshot: HashMap<u32, HashMap<&'static str, String>>,
//     /// collect → broadcast 間のバッファ
//     pub pending: Option<SnapshotFrame>,
//     /// インターバル計測用タイマー
//     pub last_sent: std::time::Instant,
// }

// impl Default for SnapshotState {
//     fn default() -> Self {
//         Self {
//             sequence: 0,
//             last_snapshot: HashMap::new(),
//             pending: None,
//             last_sent: std::time::Instant::now() - std::time::Duration::from_secs(3600),
//         }
//     }
// }

// // ============================================================================
// // コンポーネント
// // ============================================================================

// /// このコンポーネントがアタッチされたエンティティはスナップショットに含まれる
// #[derive(Component)]
// pub struct Snapshotable;

// /// スナップショットの配信先クライアントであることを示すコンポーネント
// ///
// /// `SubscribeSnapshotEvent` / `UnsubscribeSnapshotEvent` によって自動付与・除去される。
// #[derive(Component)]
// pub struct SnapshotSubscriber;

// // ============================================================================
// // イベント
// // ============================================================================

// /// クライアントがスナップショット購読を開始するイベント
// #[derive(Message)]
// pub struct SubscribeSnapshotEvent {
//     pub target: Entity,
// }

// /// クライアントがスナップショット購読を終了するイベント
// #[derive(Message)]
// pub struct UnsubscribeSnapshotEvent {
//     pub target: Entity,
// }

// /// スナップショットが送信されたときに発火するイベント
// #[derive(Message)]
// pub struct SnapshotSentEvent {
//     pub sequence: u64,
//     pub mode: SnapshotMode,
//     /// 送信先クライアント数
//     pub subscriber_count: usize,
//     pub byte_size: usize,
// }

// // ============================================================================
// // プラグイン
// // ============================================================================

// pub struct SnapshotPlugin {
//     interval_secs: f32,
//     mode: SnapshotMode,
//     registry: SnapshotRegistry,
// }

// impl Default for SnapshotPlugin {
//     fn default() -> Self {
//         Self {
//             interval_secs: 0.1,
//             mode: SnapshotMode::Delta,
//             registry: SnapshotRegistry::new(),
//         }
//     }
// }

// impl SnapshotPlugin {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     /// スナップショット送信間隔を設定する（例: 20Hz なら `0.05`）
//     pub fn interval(mut self, secs: f32) -> Self {
//         self.interval_secs = secs;
//         self
//     }

//     /// 送信モードを設定する
//     pub fn mode(mut self, mode: SnapshotMode) -> Self {
//         self.mode = mode;
//         self
//     }

//     /// スナップショット対象のコンポーネント型を登録する
//     ///
//     /// # 例
//     /// ```rust,ignore
//     /// SnapshotPlugin::new()
//     ///     .register::<Position>()
//     ///     .register::<Health>()
//     /// ```
//     pub fn register<T: SnapshotData>(mut self) -> Self {
//         self.registry.collectors.push((
//             T::component_name(),
//             Box::new(|world: &mut World| {
//                 let mut qs = world.query_filtered::<(Entity, &T), With<Snapshotable>>();
//                 qs.iter(world)
//                     .map(|(e, t)| (e.to_bits() as u32, t.to_snapshot_json()))
//                     .collect()
//             }),
//         ));
//         self
//     }
// }

// impl Plugin for SnapshotPlugin {
//     fn build(&mut self, app: &mut EcsonApp) {
//         app.insert_resource(SnapshotConfig {
//             interval_secs: self.interval_secs,
//             mode: self.mode,
//         });
//         app.insert_resource(SnapshotState::default());

//         app.add_event::<SubscribeSnapshotEvent>();
//         app.add_event::<UnsubscribeSnapshotEvent>();
//         app.add_event::<SnapshotSentEvent>();

//         app.add_systems(
//             FixedUpdate,
//             (
//                 handle_subscribe_system,
//                 handle_unsubscribe_system,
//                 collect_snapshot_system,
//                 broadcast_snapshot_system,
//             )
//                 .chain(),
//         );
//     }
// }
