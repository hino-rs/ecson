use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{LobbyCommand, LobbyMap, LobbyReadyEvent, PlayerJoinedLobbyEvent, PlayerLeftLobbyEvent};

/// 受信メッセージから LobbyCommand を解析するシステム
pub fn parse_lobby_messages_system(
    _ev_received: MessageReader<MessageReceived>,
    _ev_command: MessageWriter<LobbyCommand>,
) {
    todo!("'/lobby create <name>', '/lobby join <name>', '/lobby leave', '/lobby list' 等を解析")
}

/// ロビー作成を処理するシステム
pub fn handle_lobby_create_system(
    _ev_command: MessageReader<LobbyCommand>,
    _lobby_map: ResMut<LobbyMap>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("LobbyCommand::Create を受け取り、LobbyMap に新しいロビーを追加")
}

/// ロビー参加を処理するシステム
pub fn handle_lobby_join_system(
    _ev_command: MessageReader<LobbyCommand>,
    _lobby_map: ResMut<LobbyMap>,
    _query: Query<&mut super::InLobby>,
    _ev_joined: MessageWriter<PlayerJoinedLobbyEvent>,
    _ev_ready: MessageWriter<LobbyReadyEvent>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!(
        "LobbyCommand::Join を処理し、InLobby コンポーネントを付与。\
        max_members に達したら LobbyReadyEvent を発行"
    )
}

/// ロビー退出を処理するシステム
pub fn handle_lobby_leave_system(
    _ev_command: MessageReader<LobbyCommand>,
    _lobby_map: ResMut<LobbyMap>,
    _query: Query<(Entity, &mut super::InLobby)>,
    _ev_left: MessageWriter<PlayerLeftLobbyEvent>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("LobbyCommand::Leave を処理し、InLobby コンポーネントを除去。オーナーが抜けたらロビーを解散または引き継ぎ")
}

/// ロビー一覧を要求したクライアントへ返送するシステム
pub fn handle_lobby_list_system(
    _ev_command: MessageReader<LobbyCommand>,
    _lobby_map: Res<LobbyMap>,
    _ev_send: MessageWriter<SendMessage>,
) {
    todo!("LobbyCommand::List を処理し、公開ロビーの一覧を JSON で返送")
}

/// クライアント切断時にロビーから自動退出させるシステム
pub fn handle_lobby_disconnect_system(
    _ev_disconnected: MessageReader<crate::ecs::events::UserDisconnected>,
    _lobby_map: ResMut<LobbyMap>,
    _query: Query<(Entity, &super::InLobby)>,
    _ev_left: MessageWriter<PlayerLeftLobbyEvent>,
) {
    todo!("UserDisconnected を受け取り、ロビーから該当クライアントを削除")
}
