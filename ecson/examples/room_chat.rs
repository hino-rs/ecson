use ecson::plugins::chat::ChatFullPlugin;
use ecson::plugins::heartbeat::HeartbeatPlugin;
use ecson::prelude::*;

fn main() {
    EcsonApp::new()
        .add_plugins(HeartbeatPlugin::default().interval(30.0).timeout(60.0))
        .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run()
}
