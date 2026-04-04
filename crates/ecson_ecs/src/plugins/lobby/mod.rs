use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
mod systems;
use systems::*;

// ============================================================================
// リソース
// ============================================================================

/// 存在するロビーを管理するリソース
#[derive(Resource, Default)]
pub struct LobbyMap {
    /// lobby_name → LobbyInfo
    pub lobbies: HashMap<String, LobbyInfo>,
}

#[derive(Clone, Debug)]
pub struct LobbyInfo {
    pub name: String,
    pub owner: u64,
    pub members: Vec<u64>,
    pub max_members: u32,
    pub is_public: bool,
}

#[derive(Resource)]
pub struct LobbyConfig {
    pub default_max_members: u32,
}

// ============================================================================
// コンポーネント
// ============================================================================

/// クライアントが参加中のロビー名
#[derive(Component, Clone, Debug)]
pub struct InLobby(pub String);

// ============================================================================
// イベント（コマンド）
// ============================================================================

#[derive(Message)]
pub enum LobbyCommand {
    /// ロビー作成
    Create {
        entity: Entity,
        name: String,
        max_members: u32,
        is_public: bool,
    },
    /// ロビー参加
    Join { entity: Entity, lobby_name: String },
    /// ロビー退出
    Leave { entity: Entity },
    /// ロビー一覧取得
    List { entity: Entity },
    /// ロビー情報取得
    Info { entity: Entity, lobby_name: String },
}

// ============================================================================
// フックイベント（ユーザー向け）
// ============================================================================

#[derive(Message)]
pub struct PlayerJoinedLobbyEvent {
    pub client_id: u64,
    pub lobby_name: String,
}

#[derive(Message)]
pub struct PlayerLeftLobbyEvent {
    pub client_id: u64,
    pub lobby_name: String,
}

/// ロビーが規定人数に達してゲーム開始可能になった時に発火
#[derive(Message)]
pub struct LobbyReadyEvent {
    pub lobby_name: String,
    pub members: Vec<u64>,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct LobbyPlugin {
    pub default_max_members: u32,
}

impl Default for LobbyPlugin {
    fn default() -> Self {
        Self {
            default_max_members: 4,
        }
    }
}

impl LobbyPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_max_members(mut self, n: u32) -> Self {
        self.default_max_members = n;
        self
    }
}

impl Plugin for LobbyPlugin {
    fn build(self, app: &mut EcsonApp) {
        // LobbyConfig（新規）
        app.world.insert_resource(LobbyConfig {
            default_max_members: self.default_max_members,
        });

        if !app.world.contains_resource::<LobbyMap>() {
            app.world.insert_resource(LobbyMap::default());
        }

        // イベント登録（変更なし）
        app.add_event::<LobbyCommand>();
        app.add_event::<PlayerJoinedLobbyEvent>();
        app.add_event::<PlayerLeftLobbyEvent>();
        app.add_event::<LobbyReadyEvent>();

        app.add_systems(Update, parse_lobby_messages_system);
        app.add_systems(
            FixedUpdate,
            (
                handle_lobby_create_system,
                handle_lobby_join_system,
                handle_lobby_leave_system,
                handle_lobby_list_system,
                handle_lobby_disconnect_system,
            ),
        );
    }
}
