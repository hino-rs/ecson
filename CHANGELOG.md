# Changelog

このプロジェクトのすべての変更はこのファイルに記録されます。

フォーマットは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に準拠し、
バージョン管理は [Semantic Versioning](https://semver.org/lang/ja/) に従います。

---

## [0.1.0] - 2026-04-04

### 初回リリース

Ecson の最初の公開バージョンです。
`tokio` の非同期I/Oと `bevy_ecs` のデータ志向設計を組み合わせた、
ECS駆動のステートフル双方向サーバーフレームワークの実験的実装です。

### Added

#### コアフレームワーク

- `EcsonApp` — アプリケーションのエントリーポイント。ECSワールドとスケジューラを統合
- ECSスケジューラによる並列システム実行（ロック不要）
- `Update` / `FixedUpdate` スケジュールのサポート
- `ecson::prelude::*` による公開APIのワンストップインポート

#### ネットワーク

- `EcsonWebSocketPlugin` — WebSocket (WS) サーバー（開発・本番汎用）
- `EcsonWebSocketTlsPlugin` — TLS付きWebSocket (WSS) サーバー（PEM証明書指定）
- `EcsonWebSocketTlsDevPlugin` — 自己署名証明書を自動生成する開発用WSSサーバー
- `EcsonWebTransportDevPlugin` — WebTransport (HTTP/3 / QUIC) 開発用サーバー（低レイテンシ・データグラム通信）
- `MessageReceived` / `SendMessage` / `UserDisconnected` によるECSベースのネットワークイベントAPI

#### 組み込みプラグイン

- `HeartbeatPlugin` — Ping/Pong による死活監視と自動切断（`ClientTimedOutEvent`）
- `RateLimitPlugin` — メッセージ送信頻度の制限（Drop / Throttle / Disconnect）
- `PresencePlugin` — クライアントの在席状態管理（Online / Away / Busy）
- `SnapshotPlugin` — ECSコンポーネントの定期スナップショット送信（差分モード対応）
- `LobbyPlugin` — マッチメイキング向けロビー機能（`LobbyReadyEvent`）
- `ChatCorePlugin` — ニックネーム設定・全体ブロードキャスト
- `ChatRoomPlugin` — ルーム参加・退出・一覧取得（`ChatCorePlugin` との併用）
- `ChatFullPlugin` — Chat系機能をすべてまとめたオールインワンプラグイン
- `Spatial2DPlugin` — 2D空間の位置管理とAOI（Area of Interest）通知
- `Spatial3DFlatPlugin` — 地上系3D空間（RPG・MOBAなど）向けAOI通知
- `Spatial3DPlugin` — 完全3D空間（宇宙・飛行シムなど）向けAOI通知

#### サンプル

- `examples/echo.rs` — 接続クライアントへのエコーサーバー
- `examples/broadcast_chat.rs` — 全クライアントへのブロードキャストチャット
- `examples/room_chat.rs` — ルーム分割チャット

#### ドキュメント

- [チュートリアル](https://hino-rs.github.io/ecson/)
- [ECS解説](https://github.com/hino-rs/ecson/tree/main/docs/ECS.md)
- [CONTRIBUTING.md](./CONTRIBUTING.md)

---

> ⚠️ **本バージョンは実験段階です。** 本番環境での使用は推奨しません。APIは予告なく変更される場合があります。

[0.1.0]: https://github.com/hino-rs/ecson/releases/tag/v0.1.0