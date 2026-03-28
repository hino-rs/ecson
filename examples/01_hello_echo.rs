// v0.0.2

use fluxion::prelude::*;

// 1. システム定義
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
    // 2. FluxionAppを初期化
    let mut app = FluxionApp::default();

    // 3. FluxionAppにプラグインを追加
    // FluxionNetworkPluginでサーバーの初期化を行います
    // app.add_plugins(FluxionNetworkPlugin::new("127.0.0.1:8080")) // コア機能
    //     .add_systems(MainSchedule, echo_system);

    app.add_plugins(FluxionWebTransportPlugin::new("127.0.0.1:8080"))
        .add_systems(MainSchedule, echo_system);

    // 4. 実行
    app.run();
}
