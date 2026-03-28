// v0.0.2

use fluxion::prelude::*;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// ブロードキャストシステム
fn broadcast_system(
    mut messages: MessageReader<MessageReceived>,
    mut outbound: MessageWriter<SendMessage>,
    query: Query<(Entity, &ClientSender)>,
) {
    for event in messages.read() {
        // テキストメッセージだけを抽出（バイナリやPing等は一旦無視）
        let text_content = match &event.payload {
            Message::Text(text) => text.to_string(),
            _ => continue,
        };

        // 「誰からの発言か」分かるようにIPとポートをプレフィックスにつける
        let broadcast_text = format!("[{}]: {}", event.client_id, text_content);
        let broadcast_msg = WsMessage::Text(broadcast_text.into());

        // クエリで取得した全クライアント（Entity）に対して送信イベントを発行する
        for (target_entity, _client_id) in query.iter() {
            outbound.write(SendMessage {
                target: target_entity,
                msg: broadcast_msg.clone(),
            });
        }
    }
}

fn main() {
    let mut app = FluxionApp::new();

    app.add_plugins(FluxionNetworkPlugin::new("127.0.0.1:8080")) // コア機能
        .add_systems(MainSchedule, broadcast_system);

    app.run();
}
