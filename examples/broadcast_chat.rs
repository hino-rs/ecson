// 0.0.3

use fluxion::prelude::*;
use fluxion::plugins::chat::*;

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatCorePlugin)
        .run();
}