// v0.0.3

use fluxion::prelude::*;

// Define System 
fn echo_system(
    mut messages: MessageReader<MessageReceived>,
    mut outbound: MessageWriter<SendMessage>,
) {
    for message in messages.read() {
        outbound.write(SendMessage {
            target: message.entity,
            payload: message.payload.clone(),
        });
    }
}

fn main() {
    FluxionApp::new()
        .add_plugins((
            FluxionWebSocketPlugin::new("127.0.0.1:8080"),
            FluxionWebTransportPlugin::new("127.0.0.1:4433")
        ))
        .add_systems(Update, echo_system)
        .run();
}

// For eco-friendly
// fn main() {
//     FluxionApp::new()
//         .insert_resource(TickRate::ECO)
//         .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
//         .add_systems(Update, echo_system)
//         .run();
// }

// For real-time
// fn main() {
//     FluxionApp::new()
//         .insert_resource(TickRate::REALTIME)
//         .add_plugins(FluxionWebTransportPlugin::new("127.0.0.1:8080"))
//         .add_systems(Update, echo_system)
//         .run();
// }

