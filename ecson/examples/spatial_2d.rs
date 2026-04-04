/// 2D トップダウンゲームのサンプル
///
/// クライアントが "/move x y" を送ると、
/// interest_radius 内の近隣プレイヤー全員に座標がブロードキャストされる。
///
/// クライアント受信フォーマット: "pos {client_id} {x} {y}"
use ecson::prelude::*;
use ecson::plugins::spatial::Spatial2DPlugin;

// 接続直後のクライアントに自分の ID を通知する
fn send_hello_system(
    query: Query<(Entity, &ClientId), Added<ClientId>>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for (entity, client_id) in query.iter() {
        ev_send.write(SendMessage {
            target: entity,
            payload: NetworkPayload::Text(format!("hello {}", client_id.0)),
        });
    }
}

fn main() {
    EcsonApp::new()
        .add_plugins((
            EcsonWebSocketPlugin::new("127.0.0.1:8080"),
            EcsonWebTransportDevPlugin::new("127.0.0.1:4433")
        ))
        .add_plugins(
            Spatial2DPlugin::new()
                .interest_radius(200.0)
                .zone_size(100.0),
        )
        .add_systems(Update, send_hello_system)
        .run();
}