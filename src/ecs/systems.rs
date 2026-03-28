//! ネットワークイベントの受信処理や、クライアントへのメッセージ送信を行うECSシステム群です。

use crate::ecs::events::*;
use crate::network::channels::NetworkPayload;
use crate::{ecs::events::MessageReceived, network::channels::NetworkEvent};
use crate::ecs::components::*;
use bevy_ecs::{
    message::MessageWriter, resource::Resource, 
    system::{Commands, Query, ResMut}
};
use tokio::sync::mpsc;
use crate::ecs::resources::ConnectionMap;
use bevy_ecs::message::MessageReader;

/// Tokio側から送られてくるネットワークイベントを受信するためのリソースラッパー。
#[derive(Resource)]
pub struct NetworkReceiver(pub mpsc::Receiver<NetworkEvent>);

/// ネットワーク層からのイベントをポーリングし、ECS側の状態を更新するシステム。
/// 
/// - 新規接続: エンティティを生成して `ConnectionMap` に登録
/// - メッセージ受信: ECS側に `MessageReceived` イベントを発行
/// - 切断: `ConnectionMap` から削除し、`UserDisconnected` イベントを発行
pub fn receive_network_messages_system(
    mut commands: Commands,
    mut ecs_rx: ResMut<NetworkReceiver>,
    mut ev_msg: MessageWriter<MessageReceived>,
    mut ev_disconnect: MessageWriter<UserDisconnected>,
    mut connection_map: ResMut<ConnectionMap>,
) {
    // try_recv() により、ブロックせずに現在キューにあるイベントをすべて処理する
    while let Ok(event) = ecs_rx.0.try_recv() {
        match event {
            NetworkEvent::Connected { id, sender } => {
                // クライアントを表現するエンティティを生成し、IDと送信用チャンネルを持たせる
                let entity = commands.spawn((ClientId(id), ClientSender(sender))).id();
                connection_map.0.insert(id, entity);
                println!("ECS: 新規接続 {id} -> Entity {entity:?}");
            }
            NetworkEvent::Message { id, payload } => {
                // IDからエンティティを特定し、受信イベントを発行して他のシステムに委譲する
                if let Some(&entity) = connection_map.0.get(&id) {
                    ev_msg.write(MessageReceived { entity, client_id: id, payload });
                }
            }
            NetworkEvent::Disconnected { id } => {
                println!("ECS: {} が切断されました", id);
                // エンティティの特定とマップからの削除を同時に行い、切断イベントを発行
                if let Some(entity) = connection_map.0.remove(&id) {
                    ev_disconnect.write(UserDisconnected { entity, client_id: id });
                }
            }
        }
    }
}

/// 全クライアントに対して定期的にテストメッセージを送信するシステム。
/// （※現状は固定文字列を送るデバッグ・動作確認用の実装と思われます）
pub fn send_network_messages_system(
    query: Query<(&ClientId, &ClientSender)>,
) {
    for (client_id, sender) in query.iter() {
        let msg = NetworkPayload::Text("Hello from ECS Engine".into());

        // ECSのTick（ゲームループ）をブロックしないように try_send を使用
        if let Err(e) = sender.0.try_send(msg) {
            println!("{} への送信に失敗: {}", client_id.0, e);
        }
    }
}

/// ECS内で発行された送信要求（`SendMessage`）を処理し、ネットワーク層へ引き渡すシステム。
pub fn flush_outbound_messages_system(
    mut outbound_messages: MessageReader<SendMessage>,
    query: Query<&ClientSender>,
) {
    for outbound in outbound_messages.read() {
        // 対象エンティティの送信チャンネルを取得
        if let Ok(sender) = query.get(outbound.target) {
            // ECSの進行を妨げないよう非同期ブロックを避けて送信
            if let Err(e) = sender.0.try_send(outbound.payload.clone()) {
                eprintln!("Failed to send message to Entity {:?}: {e}", outbound.target);
            }
        } else {
            // 切断直後など、宛先エンティティが既に存在しない場合
            eprintln!("Destination Entity {:?} does not exist anymore", outbound.target);
        }
    }
}