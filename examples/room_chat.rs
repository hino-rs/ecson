// v0.0.3

use fluxion::prelude::*;

fn handle_disconnections_system(
    mut commands: Commands,
    mut ev_disconnected: MessageReader<UserDisconnected>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<&Room>,
    room_query: Query<(Entity, &Room)>,
    mut room_map: ResMut<RoomMap>,
) {
    for disconnect in ev_disconnected.read() {
        let entity = disconnect.entity;

        if let Ok(room) = client_query.get(entity) {
            // RoomMapから対象エンティティを削除
            if let Some(members) = room_map.0.get_mut(&room.0) {
                members.remove(&entity);
                
                // 同じルームの「残りのメンバー」に退室メッセージを送信
                let msg = format!("[System] User {} has left.", disconnect.client_id);
                for &target_entity in members.iter() {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(msg.clone()),
                    });
                }
            }
        }

        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}

fn chat_server_system(
    mut commands: Commands,
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_send: MessageWriter<SendMessage>,
    // メッセージ送信元のクライアント情報を取得するクエリ
    client_query: Query<(Entity, &ClientId, Option<&Room>)>,
    mut room_map: ResMut<RoomMap>,
) {
    // ECS上で発火したすべての受信メッセージを処理
    for msg in ev_received.read() {
        // テキストメッセージのみを処理対象とする
        let NetworkPayload::Text(text) = &msg.payload else {
            continue;
        };
        let sender_entity = msg.entity;
        let text = text.trim();

        // 1. ルーム参加コマンドの処理
        if let Some(room_name) = text.strip_prefix("/join") {
            let room_name = room_name.trim().to_string();

            let Ok((_, _, current_room)) = client_query.get(sender_entity) else {
                continue;
            };

            // 以前のルームにいた場合はRoomMapから削除
            if let Some(old_room) = current_room
                && let Some(members) = room_map.0.get_mut(&old_room.0)
            {
                members.remove(&sender_entity);
            }

            // 新しいルームにRoomMap上で追加
            room_map.0.entry(room_name.clone()).or_default().insert(sender_entity);

            // エンティティにRoomコンポーネントを追加(既にあれば上書きされる)
            commands.entity(sender_entity).insert(Room(room_name.clone()));

            // 本人にシステムメッセージを返信
            ev_send.write(SendMessage {
                target: sender_entity,
                payload: NetworkPayload::Text(format!("[System] Joined room: {}", room_name)),
            });
            continue;
        }

        // 2. 通常のチャットメッセージの処理
        // 送信元の現在の所属ルームを取得
        let Ok((_, client_id, current_room)) = client_query.get(sender_entity) else { continue };

        if let Some(room) = current_room {
            let broadcast_text = format!("User {}: {}", client_id.0, text);

            if let Some(members) = room_map.0.get(&room.0) {
                for &target_entity in members {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(broadcast_text.clone()),
                    });
                }
            }
        } else {
            // ルームに所属していない場合はエラーメッセージを返す
            ev_send.write(SendMessage {
                target: sender_entity,
                payload: NetworkPayload::Text(
                    "[System] You are not in a room. Type '/join <room_name>' first.".into(),
                ),
            });
        }
    }
}


fn parse_chat_messages_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_command: MessageWriter<ChatCommand>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else { continue };
        let text = text.trim();

        if let Some(room_name) = text.strip_prefix("/join ") {
            ev_command.write(ChatCommand::JoinRoom { 
                entity: msg.entity, 
                room_name: room_name.trim().to_string() 
            });
        } else if let Some(name) = text.strip_prefix("/nick ") {
            // /nick コマンドの解析
            ev_command.write(ChatCommand::Nick {
                entity: msg.entity,
                name: name.trim().to_string()
            });
        } else if text.starts_with('/') {
            // 未知のコマンドに対するエラーハンドリング
            ev_command.write(ChatCommand::Error {
                entity: msg.entity,
                message: format!("Unknown command: {}", text.split_whitespace().next().unwrap_or(text)),
            });
        } else {
            // 通常のチャット
            ev_command.write(ChatCommand::Broadcast { 
                entity: msg.entity, 
                text: text.to_string() 
            });
        }
    }
}

fn handle_join_room_system(
    mut commands: Commands,
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<Option<&Room>>,
    mut room_map: ResMut<RoomMap>,
) {
    for command in ev_command.read() {
        if let ChatCommand::JoinRoom { entity, room_name } = command {
            let Ok(current_room) = client_query.get(*entity) else { continue };

            // 古いルームからの離脱処理
            if let Some(old_room) = current_room {
                if let Some(members) = room_map.0.get_mut(&old_room.0) {
                    members.remove(entity);
                }
            }

            // 新しいルームへの参加処理
            room_map.0.entry(room_name.clone()).or_default().insert(*entity);
            commands.entity(*entity).insert(Room(room_name.clone()));
            
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[System] Joined room: {}", room_name)),
            });
        }
    }
}

fn handle_broadcast_system(
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
    // Queryに Option<&Username> を追加
    client_query: Query<(&ClientId, Option<&Username>, Option<&Room>)>,
    room_map: Res<RoomMap>,
) {
    for command in ev_command.read() {
        if let ChatCommand::Broadcast { entity, text } = command {
            let Ok((client_id, username, current_room)) = client_query.get(*entity) else { continue };

            if let Some(room) = current_room {
                // 名前が設定されていればそれを、なければ "User [ID]" を表示名にする
                let display_name = match username {
                    Some(u) => u.0.clone(),
                    None => format!("User {}", client_id.0),
                };
                
                let broadcast_text = format!("{}: {}", display_name, text);
                
                if let Some(members) = room_map.0.get(&room.0) {
                    for &target_entity in members {
                        ev_send.write(SendMessage {
                            target: target_entity,
                            payload: NetworkPayload::Text(broadcast_text.clone()),
                        });
                    }
                }
            } else {
                ev_send.write(SendMessage {
                    target: *entity,
                    payload: NetworkPayload::Text("[System] You are not in a room. Type '/join <room_name>' first.".into()),
                });
            }
        }
    }
}

fn handle_nick_system(
    mut commands: Commands,
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for command in ev_command.read() {
        if let ChatCommand::Nick { entity, name } = command {
            // Usernameコンポーネントをエンティティに付与（すでにあれば上書き）
            commands.entity(*entity).insert(Username(name.clone()));
            
            // 本人に成功メッセージを返す
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[System] Your nickname is now: {}", name)),
            });
        }
    }
}

fn handle_error_system(
    mut ev_command: MessageReader<ChatCommand>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for command in ev_command.read() {
        if let ChatCommand::Error { entity, message } = command {
            // エラー原因を本人にだけシステムメッセージとして返す
            ev_send.write(SendMessage {
                target: *entity,
                payload: NetworkPayload::Text(format!("[Error] {}", message)),
            });
        }
    }
}

fn main() {
    FluxionApp::new()
        .add_event::<ChatCommand>()
        .add_systems(MainSchedule, |mut msgs: ResMut<Messages<ChatCommand>>| msgs.update())
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_systems(
    MainSchedule,
    (
            parse_chat_messages_system,
            handle_join_room_system,
            handle_nick_system,
            handle_error_system,
            handle_broadcast_system,
            handle_disconnections_system,
        ),
        )
        .run()
}
