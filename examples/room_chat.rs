use fluxion::prelude::*;

#[derive(Component)]
struct Room(String);

fn chat_server_system(
    mut commands: Commands,
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_send: MessageWriter<SendMessage>,
    clients: Query<(Entity, &Room)>,
) {
    for msg in ev_received.read() {
        let NetworkPayload::Text(text) = &msg.payload else { continue };
        let text = text.trim();

        if let Some(room_name) = text.strip_prefix("/join ") {
            // --- 【入室処理】 ---
            let new_room = room_name.trim().to_string();
            commands.entity(msg.entity).insert(Room(new_room.clone()));
            
            ev_send.write(SendMessage {
                target: msg.entity,
                payload: NetworkPayload::Text(format!("[System] Joined: {}", new_room)),
            });
        } else if let Ok((_, room)) = clients.get(msg.entity) {
            // --- 【発言処理】 ---
            // 同じルームにいる全員を検索して送信
            for (target_entity, target_room) in clients.iter() {
                if target_room.0 == room.0 {
                    ev_send.write(SendMessage {
                        target: target_entity,
                        payload: NetworkPayload::Text(format!("[{}]: {}", room.0, text)),
                    });
                }
            }
        }
    }
}

// 切断時はエンティティを消すだけ
fn cleanup_system(mut commands: Commands, mut ev_disconnect: MessageReader<UserDisconnected>) {
    for event in ev_disconnect.read() {
        if let Ok(mut ent) = commands.get_entity(event.entity) {
            ent.despawn();
        }
    }
}

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_systems(Update, (chat_server_system, cleanup_system))
        .run()
}

// use fluxion::prelude::*;
// use fluxion::plugins::chat::*;

// fn log_joins_to_console_system(
//     mut ev_joined: MessageReader<UserJoinedRoomEvent>,
// ) {
//     for event in ev_joined.read() {
//         println!("[JOIN] User {} has joined the room '{}'.", event.client_id, event.room_name);
//     }
// }

// fn main() {
//     FluxionApp::new()
//         .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
//         .add_plugins((ChatCorePlugin, ChatRoomPlugin))
//         .add_systems(Update, log_joins_to_console_system)
//         .run()
// }