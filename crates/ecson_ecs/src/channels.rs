//! ネットワーク層（Tokio非同期タスク）とECS層（同期ゲームループ）の間で
//! やり取りされるメッセージやイベントの型を定義するモジュールです。

use tokio::sync::mpsc;

/// ネットワーク経由で送受信される実データのペイロード。
/// テキストデータ（JSONなど）とバイナリデータの両方をサポートします。
#[derive(Debug, Clone)]
pub enum NetworkPayload {
    /// 文字列データ
    Text(String),
    /// バイナリデータ
    Binary(Vec<u8>),
}

/// ネットワーク層からECS層へ送信されるイベント群。
pub enum NetworkEvent {
    /// 新しいクライアントが接続を確立したことを示します。
    Connected {
        id: u64,
        sender: mpsc::Sender<NetworkPayload>,
    },

    /// クライアントからメッセージを受信したことを示します。
    Message { id: u64, payload: NetworkPayload },

    /// クライアントとの接続が切断されたことを示します。
    Disconnected { id: u64 },
}
