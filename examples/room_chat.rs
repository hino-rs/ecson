use fluxion::prelude::*;
use fluxion::plugins::chat::ChatFullPlugin;

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run()
}