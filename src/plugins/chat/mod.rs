use bevy_ecs::prelude::*;
use crate::prelude::*;
mod systems;
use crate::plugins::chat::systems::*;

// ============================================================================
// ユーザー（アプリ開発者）に公開するフック用イベント
// ============================================================================


/// ユーザーがチャットルームに参加した時に発火するイベント
#[derive(Event, Message)]
pub struct UserJoinedRoomEvent {
    pub client_id: u64,
    pub room_name: String,
}

/// ユーザーがメッセージを送信（ブロードキャスト）した時に発火するイベント
#[derive(Event, Message)]
pub struct ChatMessageBroadcastedEvent {
    pub client_id: u64,
    pub room_name: Option<String>,
    pub text: String,
}

// ============================================================================
// プラグインの定義
// ============================================================================

// ----------------------------------------------------------------------------
// フルプラグイン
// ----------------------------------------------------------------------------

pub struct ChatFullPlugin;


impl Plugin for ChatFullPlugin {
    fn build(self, app: &mut FluxionApp) {
        // リソースの初期化
        app.world.insert_resource(RoomMap::default());
        
        // 内部イベントとフックイベントの登録
        app.add_event::<ChatCommand>();
        app.add_event::<UserJoinedRoomEvent>(); // ユーザー向けフック
        app.add_event::<ChatMessageBroadcastedEvent>(); // ユーザー向けフック

        // ボイラープレートだったシステム群をすべてエンジン側で登録！
        app.add_systems(Update, parse_chat_messages_system);
        app.add_systems(
            FixedUpdate,
            (
                handle_join_room_system,
                handle_nick_system,
                handle_list_rooms_system,
                handle_error_system,
                handle_broadcast_system,
                handle_disconnections_system, // エンジン側が提供するRoomクリーンアップなど
            ),
        );
    }
}

// ----------------------------------------------------------------------------
// コアプラグイン（パケット解析、ニックネーム、全体チャットなど最低限の機能）
// ----------------------------------------------------------------------------
pub struct ChatCorePlugin;

impl Plugin for ChatCorePlugin {
    fn build(self, app: &mut FluxionApp) {
        app.add_event::<ChatCommand>();
        app.add_event::<ChatMessageBroadcastedEvent>();
        app.add_event::<UserJoinedRoomEvent>();

        app.add_systems(Update, parse_chat_messages_system);
        app.add_systems(
            FixedUpdate,
            (
                handle_nick_system,
                handle_error_system,
                handle_broadcast_system,
            ),
        );
    }
}

// ----------------------------------------------------------------------------
// ルーム拡張プラグイン（/join, /list, RoomMapの管理など）
// ----------------------------------------------------------------------------
pub struct ChatRoomPlugin;

impl Plugin for ChatRoomPlugin {
    fn build(self, app: &mut FluxionApp) {
        // ルーム機能が追加された時だけ、RoomMapリソースが作られる
        app.world.insert_resource(RoomMap::default());
        app.add_event::<UserJoinedRoomEvent>();

        app.add_systems(
            FixedUpdate,
            (
                handle_join_room_system,
                handle_list_rooms_system,
            ),
        );
    }
}
