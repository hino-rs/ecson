// 0.0.5

use fluxion::plugins::chat::*;
use fluxion::prelude::*;

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatCorePlugin)
        .run();
}
