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
/// Tokioの非同期タスクから `mpsc::Sender` を通じてECSのリソースへ送られます。
pub enum NetworkEvent {
    /// 新しいクライアントが接続を確立したことを示します。
    Connected {
        /// クライアントに割り当てられた一意のID
        id: u64,
        /// ECS側からこのクライアントへ個別にメッセージを送信するためのチャンネル
        sender: mpsc::Sender<NetworkPayload>,
    },

    /// クライアントからメッセージを受信したことを示します。
    Message {
        /// 送信元クライアントのID
        id: u64,
        /// 受信したデータの内容
        payload: NetworkPayload,
    },

    /// クライアントとの接続が切断されたことを示します。
    Disconnected {
        /// 切断されたクライアントのID
        id: u64,
    },
}
