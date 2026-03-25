use fluxion::{ecs::resources::ServerTickRate, prelude::*};
use tokio_tungstenite::tungstenite::Message as WsMessage;

// Roomの参加・退出・メッセージ送信を処理するシステム
fn room_chat_system(
    mut commands: Commands,
    mut messages: MessageReader<MessageReceived>,
    // 1つ目のクエリ: イベントの送信元（Entity）を特定するため
    sender_query: Query<(&ClientId, Option<&Room>)>,
    // 2つ目のクエリ: メッセージの送信先（宛先）を探すため
    target_query: Query<(&ClientSender, &Room)>,
) {
    for event in messages.read() {
        let text = match &event.msg {
            WsMessage::Text(t) => t.to_string(),
            _ => continue,
        };

        // get() を使って一発でコンポーネントを取得
        let Ok((sender_id, current_room)) = sender_query.get(event.entity) else {
            // エンティティがすでに存在しない場合はスキップ
            continue; 
        };

        if text.starts_with("/join ") {
            let room_name = text.trim_start_matches("/join ").to_string();
            commands.entity(event.entity).insert(Room(room_name.clone()));
            println!("{} joined room: {}", sender_id.0, room_name);
            continue;
        }

        if text == "/leave" {
            commands.entity(event.entity).remove::<Room>();
            println!("{} left the room", sender_id.0);
            continue;
        }

        // 3. 通常のメッセージの場合、同じRoomにいる人にだけ送信する
        if let Some(room) = current_room {
            let broadcast_text = format!("[{}@{}]: {}", sender_id.0, room.0, text);
            let broadcast_msg = WsMessage::Text(broadcast_text.into());

            // target_queryは「Roomコンポーネントを持っている人」しか取得しない
            for (target_sender, target_room) in target_query.iter() {
                // 送信元のRoomと同じRoomの人にだけ送る
                if target_room.0 == room.0 {
                    let _ = target_sender.0.try_send(broadcast_msg.clone());
                }
            }
        } else {
            // Roomに入っていない人にはエラーを返す
            let warn_msg =
                WsMessage::Text("You are not in any room. Type '/join <room_name>'".into());
            // sender_queryから取得した送信元情報を元に、ClientSenderを取得して送り返す等（ここでは省略）
            println!("{} is not in a room, message ignored.", sender_id.0);
        }
    }
}

fn main() {
    println!("Starting Fluxion Room Server... 🚀");

    let mut app = FluxionApp::new();

    app.insert_resource(ServerTickRate::NORMAL)
        .add_plugins(FluxionNetworkPlugin::new("127.0.0.1:8080"))
        .add_systems(MainSchedule, room_chat_system);

    app.run();
}
