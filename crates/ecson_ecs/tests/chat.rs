use ecson_ecs::plugins::chat::ChatFullPlugin;
use ecson_ecs::plugins::chat::UserJoinedRoomEvent;
use ecson_ecs::prelude::*;
use tokio::sync::mpsc;

#[derive(Resource, Default)]
struct Inbox(Vec<SendMessage>);

// SendMessage を Inbox に吸い上げるシステム
fn collect_sends(mut ev: MessageReader<SendMessage>, mut inbox: ResMut<Inbox>) {
    for msg in ev.read() {
        inbox.0.push(SendMessage {
            target: msg.target,
            payload: msg.payload.clone(),
        });
    }
}

// テスト用クライアントエンティティをスポーンする
fn spawn_client(world: &mut World, id: u64) -> Entity {
    let (tx, _rx) = mpsc::channel(8);
    world.spawn((ClientId(id), ClientSender(tx))).id()
}

// MessageReceived を送り込む
fn send_text(world: &mut World, entity: Entity, text: &str) {
    world.write_message(MessageReceived {
        client_id: 1,
        entity,
        payload: NetworkPayload::Text(text.to_string()),
    });
}

// 送信されたメッセージのうち、特定エンティティ宛のテキストだけ返す
fn texts_for(inbox: &Inbox, target: Entity) -> Vec<String> {
    inbox
        .0
        .iter()
        .filter(|m| m.target == target)
        .filter_map(|m| match &m.payload {
            NetworkPayload::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

// テスト用アプリを組み立てる共通処理
fn build_app() -> EcsonApp {
    let mut app = EcsonApp::new();

    app.insert_resource(ConnectionMap::default());
    app.add_event::<MessageReceived>();
    app.add_event::<SendMessage>();
    app.add_event::<UserDisconnected>();
    app.add_event::<MessageSendFailed>();
    app.add_event::<UserConnected>();
    app.add_event::<SendMessage>();
    app.add_plugins(ChatFullPlugin);
    app.insert_resource(Inbox::default());
    app.add_systems(Update, collect_sends);
    app.startup();
    app
}

#[test]
fn nick_sets_username_component() {
    let mut app = build_app();
    let user = spawn_client(app.world_mut(), 1);

    send_text(app.world_mut(), user, "/join general");
    app.tick_n(10);

    let room = app
        .world()
        .get::<Room>(user)
        .expect("Roomコンポーネントがない");
    assert_eq!(room.0, "general");
}

#[test]
fn join_registers_in_room_map() {
    let mut app = build_app();
    let user = spawn_client(app.world_mut(), 1);

    send_text(app.world_mut(), user, "/join general");
    app.tick_n(10);

    let room_map = app.world().resource::<RoomMap>();
    assert!(room_map.0.get("general").is_some_and(|m| m.contains(&user)));
}

#[test]
fn join_fires_hook_event() {
    let mut app = build_app();
    let user = spawn_client(app.world_mut(), 1);

    send_text(app.world_mut(), user, "/join general");
    app.tick_once();

    let mut fired = false;
    app.world_mut()
        .resource_scope(|_world, mut messages: Mut<Messages<UserJoinedRoomEvent>>| {
            for msg in messages.drain() {
                if msg.room_name == "general" {
                    fired = true
                };
            }
        });
    assert!(fired, "UserJoinedRoomEvent が発火していない")
}

#[test]
fn join_moves_between_rooms() {
    let mut app = build_app();
    let user = spawn_client(app.world_mut(), 1);

    send_text(app.world_mut(), user, "/join general");
    app.tick_n(10);
    send_text(app.world_mut(), user, "/join random");
    app.tick_n(10);

    let room_map = app.world().resource::<RoomMap>();
    assert!(!room_map.0.get("general").is_some_and(|m| m.contains(&user)));
    assert!(room_map.0.get("random").is_some_and(|m| m.contains(&user)));
}

#[test]
fn broadcast_in_room_reaches_only_room_members() {
    let mut app = build_app();
    let user1 = spawn_client(app.world_mut(), 1);
    let user2 = spawn_client(app.world_mut(), 2);
    let user3 = spawn_client(app.world_mut(), 3);

    send_text(app.world_mut(), user1, "/join room-a");
    send_text(app.world_mut(), user2, "/join room-a");
    send_text(app.world_mut(), user3, "/join room-b");
    app.tick_n(10);

    app.world_mut().resource_mut::<Inbox>().0.clear(); // join確認メッセージを捨てる

    send_text(app.world_mut(), user1, "こんにちは");
    app.tick_n(10);

    let inbox = app.world().resource::<Inbox>();

    assert!(!texts_for(inbox, user1).is_empty(), "user1に届いていない");
    assert!(!texts_for(inbox, user2).is_empty(), "user2に届いていない");

    assert!(
        texts_for(inbox, user3).is_empty(),
        "user3に届いてしまっている"
    );
}

#[test]
fn broadcast_without_room_reaches_all_clients() {
    let mut app = build_app();

    let user1 = spawn_client(app.world_mut(), 1);
    let user2 = spawn_client(app.world_mut(), 2);

    send_text(app.world_mut(), user1, "全体メッセージ");
    app.tick_n(10);

    let inbox = app.world().resource::<Inbox>();
    assert!(
        !texts_for(inbox, user2).is_empty(),
        "全体ブロードキャストがuser2に届いていない"
    );
}

#[test]
fn disconnect_removes_from_room_map() {
    let mut app = build_app();
    let user1 = spawn_client(app.world_mut(), 1);
    let user2 = spawn_client(app.world_mut(), 2);

    send_text(app.world_mut(), user1, "/join general");
    send_text(app.world_mut(), user2, "/join general");
    app.tick_n(10);

    app.world_mut().write_message(UserDisconnected {
        entity: user1,
        client_id: 1,
    });
    app.tick_n(10);

    let room_map = app.world().resource::<RoomMap>();
    assert!(
        !room_map
            .0
            .get("general")
            .is_some_and(|m| m.contains(&user1))
    );
}

#[test]
fn disconnect_notifies_remaining_room_members() {
    let mut app = build_app();
    let user1 = spawn_client(app.world_mut(), 1);
    let user2 = spawn_client(app.world_mut(), 2);

    send_text(app.world_mut(), user1, "/join general");
    send_text(app.world_mut(), user2, "/join general");
    app.tick_n(10);
    app.world_mut().resource_mut::<Inbox>().0.clear();

    app.world_mut().write_message(UserDisconnected {
        entity: user1,
        client_id: 1,
    });
    app.tick_n(10);

    let inbox = app.world().resource::<Inbox>();

    assert!(
        texts_for(inbox, user2).iter().any(|t| t.contains("left")),
        "退出通知がuser2に届いていない"
    );
}
