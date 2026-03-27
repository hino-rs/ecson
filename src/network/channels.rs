use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use std::net::SocketAddr;

// ECSに送るデータ型
pub struct ClientMessage {
    pub client_id: SocketAddr,
    pub payload: NetworkPayload,
}

// 共通ペイロード
#[derive(Debug, Clone)]
pub enum NetworkPayload {
    Text(String),
    Binary(Vec<u8>),
}

// ネットワーク層からECSへ送るイベント
pub enum NetworkEvent {
    Connected {
        id: u64,
        sender: mpsc::Sender<NetworkPayload>,
    },
    Message {
        id: u64,
        payload: NetworkPayload,
    },
    Disconnected {
        id: u64,
    }
}