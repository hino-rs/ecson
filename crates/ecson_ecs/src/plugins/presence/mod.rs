use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::{collections::HashMap, fmt};
mod systems;
use systems::*;

// ============================================================================
// リソース
// ============================================================================

/// オンライン中のクライアント一覧を管理するリソース
#[derive(Resource, Default)]
pub struct PresenceMap {
    /// client_id → ステータス
    pub map: HashMap<u64, PresenceStatus>,
}

// ============================================================================
// コンポーネント
// ============================================================================

/// クライアントエンティティの在席状態を表すコンポーネント
#[derive(Component, Clone, Debug, PartialEq, Eq, Default)]
pub enum PresenceStatus {
    #[default]
    Online,
    Away,
    Busy,
}

impl fmt::Display for PresenceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PresenceStatus::Online => write!(f, "online"),
            PresenceStatus::Away => write!(f, "away"),
            PresenceStatus::Busy => write!(f, "busy"),
        }
    }
}

// ============================================================================
// イベント
// ============================================================================

/// クライアントの在席状態が変化したときに発火
#[derive(Message)]
pub struct PresenceChangedEvent {
    pub client_id: u64,
    pub entity: Entity,
    pub status: PresenceStatus,
}

// ============================================================================
// プラグイン
// ============================================================================

pub struct PresencePlugin;

impl Plugin for PresencePlugin {
    fn build(self, app: &mut EcsonApp) {
        if !app.world.contains_resource::<PresenceMap>() {
            app.world.insert_resource(PresenceMap::default());
        }

        app.add_event::<PresenceChangedEvent>();

        app.add_systems(Update, parse_presence_messages_system);
        app.add_systems(
            FixedUpdate,
            (
                handle_presence_update_system,
                handle_presence_disconnect_system,
            ),
        );
    }
}
