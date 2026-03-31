use crate::prelude::*;
use crate::plugins::chat::UserJoinedRoomEvent;

/// チャット機能固有のコマンドを処理するためのイベント群。
#[derive(Event, Message)]
pub enum ChatCommand {
    /// ルームへの入室要求
    JoinRoom { entity: Entity, room_name: String },
    /// ニックネームの変更要求
    Nick { entity: Entity, name: String },
    /// 存在するルーム一覧の取得要求
    ListRooms { entity: Entity },
    /// ルーム内（または全体）へのテキストブロードキャスト要求
    Broadcast { entity: Entity, text: String },
    /// エラーメッセージの通知（システムからクライアントへエラーを返す際などに使用）
    Error { entity: Entity, message: String },
}

/// 受信したテキストメッセージを解析し、適切な ChatCommand イベントを発行するシステム
pub fn parse_chat_messages_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_command: MessageWriter<ChatCommand>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else { continue };
        let text = text.trim();

        let command = if let Some(room_name) = text.strip_prefix("/join ") {
            ChatCommand::JoinRoom { entity: msg.entity, room_name: room_name.trim().to_string() }
        } else if let Some(name) = text.strip_prefix("/nick ") {
            ChatCommand::Nick { entity: msg.entity, name: name.trim().to_string() }
        } else if text == "/list" {
            ChatCommand::ListRooms { entity: msg.entity }
        } else if text.starts_with('/') {
            let unknown_cmd = text.split_whitespace().next().unwrap_or(text);
            ChatCommand::Error { entity: msg.entity, message: format!("Unknown command: {unknown_cmd}") }
        } else {
            ChatCommand::Broadcast { entity: msg.entity, text: text.to_string() }
        };

        ev_command.write(command);
    }
}

pub fn handle_join_room_system(
    mut commands: Commands,
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
    mut ev_hook_joined: MessageWriter<UserJoinedRoomEvent>,
    client_query: Query<(&ClientId, Option<&Room>)>,
    mut room_map: ResMut<RoomMap>,
) {
    for command in ev_command.read() {
        let ChatCommand::JoinRoom { entity, room_name } = command else { continue };
        let Ok((client_id, current_room_opt)) = client_query.get(*entity) else { continue };

        // 古いルームからの離脱処理
        if let Some(old_room) = current_room_opt
            && let Some(members) = room_map.0.get_mut(&old_room.0) {
                members.remove(entity);
        }

        // 新しいルームへの参加処理
        room_map.0.entry(room_name.clone()).or_default().insert(*entity);
        commands.entity(*entity).insert(Room(room_name.clone()));

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!("[System] Joined room: {room_name}")),
        });

        ev_hook_joined.write(UserJoinedRoomEvent {
            client_id: client_id.0,
            room_name: room_name.clone(),
        });
    }
}

pub fn handle_list_rooms_system(
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
    room_map: Res<RoomMap>,
) {
    for command in ev_command.read() {
        let ChatCommand::ListRooms { entity } = command else { continue };
        
        let mut list_text = String::from("[System] Active Rooms:\n");
        let mut has_active_rooms = false;

        for (room_name, members) in room_map.0.iter().filter(|(_, m)| !m.is_empty()) {
            list_text.push_str(&format!("  - {room_name} ({} users)\n", members.len()));
            has_active_rooms = true;
        }

        if !has_active_rooms {
            list_text.push_str("  No active rooms right now.");
        }

        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(list_text),
        });
    }
}

pub fn handle_error_system(
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for command in ev_command.read() {
        let ChatCommand::Error { entity, message } = command else { continue };
        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!("[Error] {message}")),
        });
    }
}

pub fn handle_nick_system(
    mut commands: Commands,
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for command in ev_command.read() {
        let ChatCommand::Nick { entity, name } = command else { continue };
        commands.entity(*entity).insert(Username(name.clone()));
        
        ev_send.write(SendMessage {
            target: *entity,
            payload: NetworkPayload::Text(format!("[System] Your nickname is now: {name}")),
        });
    }
}

/// チャットの送信を処理するシステム。
/// RoomMapリソースが存在するかどうか（ChatRoomPluginが読み込まれているか）で挙動が自動的に変わります。
pub fn handle_broadcast_system(
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
    // 自身の情報（ルームに入っていればRoomコンポーネントが取れる）
    client_query: Query<(Entity, &ClientId, Option<&Username>, Option<&Room>)>,
    // 送信先一覧を取得するためのクエリ（全員）
    all_clients_query: Query<Entity, With<ClientId>>,
    room_map_opt: Option<Res<RoomMap>>, 
) {
    for command in ev_command.read() {
        let ChatCommand::Broadcast { entity, text } = command else { continue };
        let Ok((_, client_id, username, current_room)) = client_query.get(*entity) else { continue };

        let display_name = username
            .map(|u| u.0.clone())
            .unwrap_or_else(|| format!("User {}", client_id.0));
        let broadcast_text = format!("{display_name}: {text}");

        // ルームプラグインが有効 ＆ ユーザーがルームに所属している場合
        if let (Some(room_map), Some(room)) = (&room_map_opt, current_room) {
            if let Some(members) = room_map.0.get(&room.0) {
                for &target_entity in members {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(broadcast_text.clone()),
                    });
                }
            }
        } 
        // ルームプラグインが無効、またはユーザーがルームに入っていない場合（全体チャット）
        else {
            for target_entity in all_clients_query.iter() {
                ev_send.write(SendMessage {
                    target: target_entity,
                    payload: NetworkPayload::Text(broadcast_text.clone()),
                });
            }
        }
    }
}


pub fn handle_disconnections_system(
    mut ev_disconnected: MessageReader<UserDisconnected>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<&Room>,
    room_map: Res<RoomMap>,
) {
    for disconnect in ev_disconnected.read() {
        // 切断したユーザーがルームに所属していた場合のみ処理
        let Ok(room) = client_query.get(disconnect.entity) else { continue };
        let Some(members) = room_map.0.get(&room.0) else { continue };

        let msg = format!("[System] User {} has left.", disconnect.client_id);
        
        for &target_entity in members {
            // 自分自身には送らない
            if target_entity != disconnect.entity {
                ev_send.write(SendMessage {
                    target: target_entity,
                    payload: NetworkPayload::Text(msg.clone()),
                });
            }
        }
    }
}