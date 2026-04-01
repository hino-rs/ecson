use ecson::prelude::*;
use ecson::plugins::chat::ChatFullPlugin;

use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("trace")
    ).init();

    EcsonApp::new()
        .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run()
}