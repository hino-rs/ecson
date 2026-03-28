//! クライアントとの通信や、システム間でやり取りされるイベント（メッセージ）を定義します。

use bevy_ecs::{entity::Entity, event::Event};
use bevy_ecs::message::Message;
use crate::network::channels::NetworkPayload;

/// クライアントからメッセージを受信した際に発行されるイベント。
/// ネットワーク層からECS層へデータが渡されたことをシステムに通知します。
#[derive(Event, Message)]
pub struct MessageReceived {
    /// メッセージを送信したクライアントに対応するECSエンティティ
    pub entity: Entity,
    /// クライアントのネットワークID
    pub client_id: u64,
    /// 受信したデータ本体
    pub payload: NetworkPayload,
}

/// サーバーから特定のクライアント（Entity）へメッセージを送信するためのイベント。
/// システムがこのイベントを発行すると、ネットワーク送信処理によってクライアントへ届けられます。
#[derive(Message)]
pub struct SendMessage {
    /// 送信先のクライアントに対応するECSエンティティ
    pub target: Entity,
    /// 送信するデータ本体
    pub payload: NetworkPayload,
}

/// 接続している全クライアントに対して一斉にメッセージを送信するためのイベント。
#[derive(Message)]
pub struct BroadcastMessage {
    /// 一斉送信するデータ本体
    pub msg: NetworkPayload,
}

/// クライアントとの接続が切断された際に発行されるイベント。
/// 退出処理やリソースのクリーンアップ（ConnectionMapからの削除など）に使用されます。
#[derive(Event, Message)]
pub struct UserDisconnected {
    /// 切断されたクライアントに対応するECSエンティティ
    pub entity: Entity,
    /// 切断されたクライアントのネットワークID
    pub client_id: u64,
}

/// チャット機能固有のコマンドを処理するためのイベント群。
#[derive(Event, Message)]
pub enum ChatCommand {
    /// ルームへの入室要求
    JoinRoom { entity: Entity, room_name: String },
    /// ニックネームの変更要求
    Nick { entity: Entity, name: String },
    /// 存在するルーム一覧の取得要求
    ListRooms { entity: Entity },
    /// ルーム内（または全体）へのテキストブロードキャスト要求
    Broadcast { entity: Entity, text: String },
    /// エラーメッセージの通知（システムからクライアントへエラーを返す際などに使用）
    Error { entity: Entity, message: String },
}