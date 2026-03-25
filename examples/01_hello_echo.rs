use fluxion::prelude::*;

fn echo_system(
    mut messages: MessageReader<MessageReceived>,
    query: Query<(&ClientId, &ClientSender)>,
) {
    for event in messages.read() {
        for (client_id, sender) in query.iter() {
            if client_id.0 == event.client_id {
                let _ = sender.0.try_send(event.msg.clone());
            }
        }
    }
}

fn main() {
    let mut app = FluxionApp::default();

    app.add_plugins(FluxionNetworkPlugin::new("127.0.0.1:8080")) // コア機能
        .add_systems(MainSchedule, echo_system);

    app.run();
}