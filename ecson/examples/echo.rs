use ecson::prelude::*;

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
    tracing_subscriber::fmt::init();

    EcsonApp::new()
        .add_plugins((
            EcsonWebSocketPlugin::new("127.0.0.1:80800"),
            EcsonWebTransportDevPlugin::new("127.0.0.1:4433"),
        ))
        .add_systems(Update, echo_system)
        .run();
}

// For eco-friendly
// fn main() {
//     EcsonApp::new()
//         .insert_resource(TickRate::ECO)
//         .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
//         .add_systems(Update, echo_system)
//         .run();
// }

// For real-time
// fn main() {
//     EcsonApp::new()
//         .insert_resource(TickRate::REALTIME)
//         .add_plugins(EcsonWebTransportPlugin::new("127.0.0.1:8080"))
//         .add_systems(Update, echo_system)
//         .run();
// }
