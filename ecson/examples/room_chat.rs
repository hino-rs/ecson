use ecson::plugins::chat::ChatFullPlugin;
use ecson::prelude::*;

fn main() {
    EcsonApp::new()
        .add_plugins((
            EcsonWebSocketPlugin::new("127.0.0.1:8080"),
            EcsonWebTransportDevPlugin::new("127.0.0.1:4433"),
        ))
        .add_plugins(ChatFullPlugin)
        .run()
}
