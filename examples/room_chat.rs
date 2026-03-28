// v0.0.3

use fluxion::prelude::*;

fn handle_disconnections_system(
    mut commands: Commands,
    mut ev_disconnected: MessageReader<UserDisconnected>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<&Room>,
    room_query: Query<(Entity, &Room)>,
) {
    for disconnect in ev_disconnected.read() {
        let entity = disconnect.entity;
        let client_id = disconnect.client_id;

        // 対象のエンティティがルームに所属していたか確認
        if let Ok(room) = client_query.get(entity) {
            let msg = format!("[System] User {} has left the room.", client_id);
            
            // 同じルームの他のユーザーに退室メッセージをブロードキャスト
            for (target_entity, target_room) in room_query.iter() {
                if target_room.0 == room.0 && target_entity != entity {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(msg.clone()),
                    });
                }
            }
        }

        // 最後にエンティティを完全に削除（クリーンアップ）
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
            println!("Cleaned up entity for client {}", client_id);
        }
    }
}

fn chat_server_system(
    mut commands: Commands,
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_send: MessageWriter<SendMessage>,
    // メッセージ送信元のクライアント情報を取得するクエリ
    client_query: Query<(Entity, &ClientId, Option<&Room>)>,
    // ルームに所属している全クライアントを取得するクエリ
    room_query: Query<(Entity, &Room)>,
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

            // エンティティにRoomコンポーネントを追加(既にあれば上書きされる)
            commands.entity(sender_entity).insert(Room(room_name.clone()));

            // 本人にシステムメッセージを返信
            ev_send.write(SendMessage {
                target: sender_entity,
                payload: NetworkPayload::Text(format!("[System] Joined room: {}", room_name)),
            });

            println!("Client {} joined room: {}", msg.client_id, room_name);
            continue;
        }

        // 2. 通常のチャットメッセージの処理
        // 送信元の現在の所属ルームを取得
        let Ok((_, client_id, current_room)) = client_query.get(sender_entity) else { continue };

        if let Some(room) = current_room {
            // ルームに所属している場合、同じルームにいる全員(自分を含む)にブロードキャスト
            let broadcast_text = format!("User {}: {}", client_id.0, text);

            for (target_entity, target_room) in room_query.iter() {
                if target_room.0 == room.0 {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(broadcast_text.clone()),
                    });
                }
            }
            println!("[Room {}] {}", room.0, broadcast_text);
        } else {
            // ルームに所属していない場合はエラーメッセージを返す
            ev_send.write(SendMessage {
                target: sender_entity,
                payload: NetworkPayload::Text("[System] You are not in a room. Type '/join <room_name>' first.".into()),
            });
        }
    }
}

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_systems(MainSchedule, (chat_server_system, handle_disconnections_system))
        .run()
}
