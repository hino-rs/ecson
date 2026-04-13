use axum::{Router, routing::get};
use ecson::prelude::*;

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

async fn root() -> &'static str {
    "Hello, Axum!"
}

fn main() {
    tracing_subscriber::fmt::init();
    let router = Router::new().route("/", get(root));

    EcsonApp::new()
        .add_plugins((
            EcsonWebSocketPlugin::new("127.0.0.1:8080"),
            EcsonWebTransportDevPlugin::new("127.0.0.1:4433"),
        ))
        .add_plugins(EcsonHttpPlugin::new("127.0.0.1:8081").router(router))
        .add_systems(Update, echo_system)
        .run();
}
