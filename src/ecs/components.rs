use bevy_ecs::prelude::Component;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use std::net::SocketAddr;


/// クライアントを一意に識別するためのIDコンポーネント
/// 接続時にインクリメントされるカウンターやUUID
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);

/// (同期・Tick駆動) → (非同期・ネットワークI/O)へ
/// メッセージを送信するためのブリッジ
#[derive(Component)]
pub struct WsSender {
    // tungsteniteの生型を隠蔽し、シンプルにStringを送れるようにする
    pub tx: mpsc::UnboundedSender<String>,
}

impl WsSender {
    /// ユーザーがシステム内で簡単にメッセージを送れるようにするヘルパー
    pub fn send(&self, message: impl Into<String>) {
        // Tokio側がすでに切断されて受信口が閉じていた場合はエラーを無視にする
        let _ = self.tx.send(message.into());
    }
}

// エンティティに持たせる送信用のポスト
#[derive(Component)]
pub struct ClientSender(pub mpsc::Sender<Message>);

// エンティティに持たせるクライアントID
#[derive(Component)]
pub struct ClientId(pub SocketAddr);

// クライアントが所属しているルームを表す
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Room(pub String);