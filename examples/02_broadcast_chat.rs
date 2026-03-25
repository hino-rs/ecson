use fluxion::plugin::FluxionNetworkPlugin;
use fluxion::prelude::*;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// ブロードキャストシステム
fn broadcast_system(
    mut messages: MessageReader<MessageReceived>,
    query: Query<(&ClientId, &ClientSender)>,
) {
    for event in messages.read() {
        // テキストメッセージだけを抽出（バイナリやPing等は一旦無視）
        let text_content = match &event.msg {
            WsMessage::Text(text) => text.to_string(),
            _ => continue,
        };

        // 「誰からの発言か」分かるようにIPとポートをプレフィックスにつける
        let broadcast_text = format!("[{}]: {}", event.client_id, text_content);
        let broadcast_msg = WsMessage::Text(broadcast_text.into());

        // クエリで取得した「全クライアント」に対してメッセージを送信
        for (target_client_id, sender) in query.iter() {
            if let Err(e) = sender.0.try_send(broadcast_msg.clone()) {
                println!("{} への送信に失敗: {}", target_client_id.0, e);
            }
        }
    }
}

fn main() {
    let mut app = FluxionApp::new();

    app.add_plugins(FluxionNetworkPlugin::new("127.0.0.1:8080")) // コア機能
        .add_systems(MainSchedule, broadcast_system);

    app.run();
}
