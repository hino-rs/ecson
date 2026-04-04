//! クライアント（接続ユーザー）の属性や状態を表すECSコンポーネント群を定義します。

use crate::channels::NetworkPayload;
use bevy_ecs::prelude::*;
use tokio::sync::mpsc;

/// ネットワーク層へデータを送信するためのチャンネルを保持するコンポーネント。
#[derive(Component)]
pub struct ClientSender(pub mpsc::Sender<NetworkPayload>);

/// クライアントを一意に識別するネットワークIDコンポーネント。
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

/// クライアントが現在所属しているルーム名を表すコンポーネント。
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Room(pub String);

/// クライアントの表示名（ニックネーム）を保持するコンポーネント。
#[derive(Component)]
pub struct Username(pub String);
