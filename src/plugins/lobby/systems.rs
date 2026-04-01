use bevy_ecs::prelude::*;
use crate::prelude::*;
use super::{
    InLobby, LobbyCommand, LobbyConfig, LobbyInfo, LobbyMap,
    LobbyReadyEvent, PlayerJoinedLobbyEvent, PlayerLeftLobbyEvent,
};

// ============================================================================
// パースシステム
// ============================================================================

/// 受信メッセージから LobbyCommand を解析するシステム
///
/// # コマンド仕様
/// ```text
/// /lobby create <name> [max_members] [private]
/// /lobby join <name>
/// /lobby leave
/// /lobby list
/// /lobby info <name>
/// ```
///
/// # スケジュール: Update
pub fn parse_lobby_messages_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_command: MessageWriter<LobbyCommand>,
    config: Res<LobbyConfig>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else { continue };
        let Some(rest) = text.trim().strip_prefix("/lobby ") else { continue };
        let parts: Vec<&str> = rest.split_whitespace().collect();

        let command = match parts.as_slice() {
            ["create", name, tail @ ..] => {
                let max_members = tail
                    .first()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(config.default_max_members);
                let is_public = !tail.get(1).is_some_and(|s| *s == "private");
                LobbyCommand::Create {
                    entity: msg.entity,
                    name: (*name).to_string(),
                    max_members,
                    is_public,
                }
            }
            ["join", name] => LobbyCommand::Join {
                entity: msg.entity,
                lobby_name: (*name).to_string(),
            },
            ["leave"] => LobbyCommand::Leave { entity: msg.entity },
            ["list"]  => LobbyCommand::List  { entity: msg.entity },
            ["info", name] => LobbyCommand::Info {
                entity: msg.entity,
                lobby_name: (*name).to_string(),
            },
            _ => continue,
        };

        ev_command.write(command);
    }
}

// ============================================================================
// ハンドラシステム
// ============================================================================

/// ロビー作成を処理するシステム
///
/// 作成者は自動的にロビーへ参加し、`InLobby` コンポーネントが付与される。
/// 同名ロビーが既に存在する場合、またはすでに別ロビーに参加中の場合はエラーを返す。
///
/// # スケジュール: FixedUpdate
pub fn handle_lobby_create_system(
    mut commands: Commands,
    mut ev_command: MessageReader<LobbyCommand>,
    mut lobby_map: ResMut<LobbyMap>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<(&ClientId, Option<&InLobby>)>,
) {
    for command in ev_command.read() {
        let LobbyCommand::Create { entity, name, max_members, is_public } = command else {
            continue;
        };
        let Ok((client_id, current_lobby)) = client_query.get(*entity) else { continue };

        // すでにロビーに参加中
        if current_lobby.is_some() {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(
                    "[Lobby] Leave your current lobby before creating one.".to_string(),
                ),
            });
            continue;
        }

        // 同名ロビーが既に存在する
        if lobby_map.lobbies.contains_key(name.as_str()) {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[Lobby] '{}' already exists.", name)),
            });
            continue;
        }

        lobby_map.lobbies.insert(
            name.clone(),
            LobbyInfo {
                name: name.clone(),
                owner: client_id.0,
                members: vec![client_id.0],
                max_members: *max_members,
                is_public: *is_public,
            },
        );
        commands.entity(*entity).insert(InLobby(name.clone()));

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!(
                "[Lobby] Created and joined '{}' (1/{}).",
                name, max_members
            )),
        });
    }
}

/// ロビー参加を処理するシステム
///
/// 満員になった瞬間に `LobbyReadyEvent` を発行する。
///
/// # スケジュール: FixedUpdate
pub fn handle_lobby_join_system(
    mut commands: Commands,
    mut ev_command: MessageReader<LobbyCommand>,
    mut lobby_map: ResMut<LobbyMap>,
    mut ev_joined: MessageWriter<PlayerJoinedLobbyEvent>,
    mut ev_ready: MessageWriter<LobbyReadyEvent>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<(&ClientId, Option<&InLobby>)>,
) {
    for command in ev_command.read() {
        let LobbyCommand::Join { entity, lobby_name } = command else { continue };
        let Ok((client_id, current_lobby)) = client_query.get(*entity) else { continue };

        // すでに別のロビーに参加中
        if current_lobby.is_some() {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(
                    "[Lobby] Leave your current lobby before joining another.".to_string(),
                ),
            });
            continue;
        }

        let Some(lobby) = lobby_map.lobbies.get_mut(lobby_name.as_str()) else {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[Lobby] '{}' not found.", lobby_name)),
            });
            continue;
        };

        if lobby.members.len() as u32 >= lobby.max_members {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[Lobby] '{}' is full.", lobby_name)),
            });
            continue;
        }

        lobby.members.push(client_id.0);
        let count = lobby.members.len() as u32;
        let max   = lobby.max_members;
        let members_snapshot = lobby.members.clone();

        commands.entity(*entity).insert(InLobby(lobby_name.clone()));

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!(
                "[Lobby] Joined '{}' ({}/{}).",
                lobby_name, count, max
            )),
        });

        ev_joined.write(PlayerJoinedLobbyEvent {
            client_id: client_id.0,
            lobby_name: lobby_name.clone(),
        });

        // 満員になったらゲーム開始イベントを発行
        if count >= max {
            ev_ready.write(LobbyReadyEvent {
                lobby_name: lobby_name.clone(),
                members: members_snapshot,
            });
        }
    }
}

/// ロビー退出を処理するシステム
///
/// - オーナーが退出した場合は次のメンバーへ所有権を移譲する
/// - メンバーが 0 になった場合はロビーを解散する
///
/// # スケジュール: FixedUpdate
pub fn handle_lobby_leave_system(
    mut commands: Commands,
    mut ev_command: MessageReader<LobbyCommand>,
    mut lobby_map: ResMut<LobbyMap>,
    mut ev_left: MessageWriter<PlayerLeftLobbyEvent>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<(&ClientId, Option<&InLobby>)>,
) {
    for command in ev_command.read() {
        let LobbyCommand::Leave { entity } = command else { continue };
        let Ok((client_id, current_lobby)) = client_query.get(*entity) else { continue };

        let Some(InLobby(lobby_name)) = current_lobby else {
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text("[Lobby] You are not in a lobby.".to_string()),
            });
            continue;
        };
        let lobby_name = lobby_name.clone();

        leave_lobby(&mut lobby_map, &lobby_name, client_id.0);
        commands.entity(*entity).remove::<InLobby>();

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!("[Lobby] Left '{}'.", lobby_name)),
        });
        ev_left.write(PlayerLeftLobbyEvent {
            client_id: client_id.0,
            lobby_name,
        });
    }
}

/// ロビー一覧を要求したクライアントへ返送するシステム
///
/// 公開ロビーのみ一覧表示する。
///
/// # スケジュール: FixedUpdate
pub fn handle_lobby_list_system(
    mut ev_command: MessageReader<LobbyCommand>,
    lobby_map: Res<LobbyMap>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for command in ev_command.read() {
        let LobbyCommand::List { entity } = command else { continue };

        let public: Vec<_> = lobby_map.lobbies.values().filter(|l| l.is_public).collect();
        let mut text = String::from("[Lobby] Public lobbies:\n");

        if public.is_empty() {
            text.push_str("  (none)");
        } else {
            for l in &public {
                text.push_str(&format!(
                    "  {} ({}/{})  owner:{}\n",
                    l.name,
                    l.members.len(),
                    l.max_members,
                    l.owner,
                ));
            }
        }

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(text),
        });
    }
}

/// クライアント切断時にロビーから自動退出させるシステム
///
/// `InLobby` を持つエンティティが切断された場合、ロビーから除去して
/// `PlayerLeftLobbyEvent` を発行する。
/// Commands を使わないのは、切断時にはエンティティの despawn が
/// ネットワーク層またはハートビートで既に行われるため。
///
/// # スケジュール: FixedUpdate
pub fn handle_lobby_disconnect_system(
    mut ev_disconnected: MessageReader<crate::ecs::events::UserDisconnected>,
    mut lobby_map: ResMut<LobbyMap>,
    query: Query<(Entity, &InLobby)>,
    mut ev_left: MessageWriter<PlayerLeftLobbyEvent>,
) {
    for ev in ev_disconnected.read() {
        // 切断したエンティティが属するロビーを特定
        let Some(lobby_name) = query
            .iter()
            .find(|(e, _)| *e == ev.entity)
            .map(|(_, l)| l.0.clone())
        else {
            continue; // どのロビーにも属していない
        };

        leave_lobby(&mut lobby_map, &lobby_name, ev.client_id);

        ev_left.write(PlayerLeftLobbyEvent {
            client_id: ev.client_id,
            lobby_name,
        });
    }
}

// ============================================================================
// 内部ヘルパー
// ============================================================================

/// メンバー除去・オーナー移譲・空ロビー解散をまとめた共通処理
fn leave_lobby(lobby_map: &mut LobbyMap, lobby_name: &str, client_id: u64) {
    let Some(lobby) = lobby_map.lobbies.get_mut(lobby_name) else { return };

    lobby.members.retain(|&id| id != client_id);

    if lobby.members.is_empty() {
        lobby_map.lobbies.remove(lobby_name);
    } else if lobby.owner == client_id {
        // 次のメンバーへ所有権を移譲
        lobby.owner = lobby.members[0];
    }
}