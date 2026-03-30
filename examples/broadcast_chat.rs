use fluxion::prelude::*;

fn broadcast_system(
    mut ev_received: MessageReader<MessageReceived>,
    mut ev_send: MessageWriter<SendMessage>,
    client_query: Query<Entity, With<ClientId>>,
) {
    for msg in ev_received.read() {
        // テキストメッセージのみを処理対象とする
        let NetworkPayload::Text(text) = &msg.payload else { 
            continue; 
        };

        // 「誰が発言したか」を分かりやすくフォーマット
        let broadcast_text = format!("User {}: {}", msg.client_id, text);
        let payload = NetworkPayload::Text(broadcast_text);

        // クエリで取得した全クライアント（エンティティ）に対してSendMessageイベントを発行
        for target_entity in client_query.iter() {
            ev_send.write(SendMessage {
                target: target_entity,
                payload: payload.clone(),
            });
        }
    }
}


fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_systems(Update, broadcast_system)
        .run();
}


// 0.0.5

// use fluxion::plugins::chat::*;
// use fluxion::prelude::*;

// fn main() {
//     FluxionApp::new()
//         .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
//         .add_plugins(ChatCorePlugin)
//         .run();
// }
