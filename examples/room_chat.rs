// v0.0.5

use fluxion::prelude::*;
use fluxion::plugins::chat::*;

fn log_joins_to_console_system(
    mut ev_joined: MessageReader<UserJoinedRoomEvent>,
) {
    for event in ev_joined.read() {
        println!("[JOIN] User {} has joined the room '{}'.", event.client_id, event.room_name);
    }
}

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins((ChatCorePlugin, ChatRoomPlugin))
        .add_systems(Update, log_joins_to_console_system)
        .run()
}