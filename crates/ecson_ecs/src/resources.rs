//! アプリケーション全体で共有されるECSリソースを定義します。

use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use crate::channels::NetworkEvent;

/// ネットワーク接続ID（`u64`）から対応する `Entity` をO(1)で検索するための内部リソース。
#[derive(Resource, Default)]
pub struct ConnectionMap(pub HashMap<u64, Entity>);

/// ルーム名（`String`）から、そのルームに参加している `Entity` の一覧をO(1)で検索するためのリソース。
#[derive(Resource, Default)]
pub struct RoomMap(pub HashMap<String, HashSet<Entity>>);

/// ECSのWorldを経由して、Tokio側へイベントを送るためのチャンネル（Sender）を保持するリソース。
#[derive(Resource)]
pub struct NetworkSender(pub mpsc::Sender<NetworkEvent>);
