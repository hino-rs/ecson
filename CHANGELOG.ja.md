# 変更履歴

このプロジェクトの主要な変更はすべてこのファイルに記録されます。

フォーマットは [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) に基づき、
バージョン管理は [Semantic Versioning](https://semver.org/) に準拠しています。

---

## [0.2.4] - 2026-04-08

### 修正

#### マクロ

- `#[component]`、`#[resource]`、`#[message]` がユーザーの crate に `bevy_ecs` 直接依存なしで動作するよう修正
  - `bevy_ecs` の derive macro はコンパイル後の生成コードに `::bevy_ecs::...` をハードコードするため、ユーザーが `bevy_ecs` を直接依存に持たない場合にコンパイルエラーになっていた
  - `ecson_macros` 側で impl を手動生成し、生成コードのパスをすべて `::ecson::bevy_ecs::...` 経由に変更
  - `ecson` が `#[doc(hidden)]` で `bevy_ecs` を再エクスポートすることで上記パスを解決可能に

#### Ecson

- `#[ecson::component]` / `#[ecson::resource]` / `#[ecson::message]` の絶対パス構文に対応
  - `pub use ecson_macros::{component, resource, message};` をクレートルートに追加

---

## [0.2.3]

`ecson::prelude`の更新

## [0.2.2]

`ecson::prelude`の更新

---

## [0.2.1]

ファサードクレートの公開APIの整理

### 追加

#### コアフレームワーク

- `Plugins` と `PluginsState` を `ecson::prelude` からエクスポートするようにしました
- `Shutdown` と `ShutdownFlag` を `ecson::prelude` からエクスポートするようにしました
  （v0.2.0 で追加されていましたが、prelude に含まれていませんでした）

### 変更

#### コアフレームワーク

- `ecson/src/lib.rs`: 内部クレート（`ecson_core`・`ecson_ecs`・`ecson_network`）の
  bare `pub use` を削除し、実装詳細の公開APIへのリークを防ぎました
- `bevy_ecs` の再エクスポートに `#[doc(hidden)]` を付与しました。
  高度なユースケース向けには引き続き利用可能ですが、公開ドキュメントには表示されません
- クレートレベルの doc コメントを追加し、docs.rs のランディングページを整備しました
  
---

## [0.2.0] - 2026-04-06

ライフサイクルの改良

### 追加

#### コアフレームワーク

- サーバー終了時に一度だけ実行される`Shutdown`スケジュール
- ライフサイクルの終了を判定するための`ShutdownFlag`

#### 組み込みプラグイン

- シャットダウン時のクリーンアップ処理用に`Plugin::cleanup`
- Plugin::build のシグネチャを `self` から `&self` に変更
- EcsonApp 内でプラグインを `Vec<Box<dyn Plugin>>` として格納するように変更
- メインループ終了後かつ Shutdown スケジューリング前に、すべてのプラグインに対して cleanup を呼び出すように変更

---

## [0.1.1] - 2026-04-06

### 追加

#### コアフレームワーク

`EcsonApp` に以下のメソッドを追加しました：

- `startup`: `Startup` を一度だけ実行します。
- `tick_once`: `Update` と `FixedUpdate` を一度だけ実行します。
- `tick_n(n)`: `Update` と `FixedUpdate` を n 回実行します。

#### ドキュメント

- [CHANGELOG.ja.md](./CHANGELOG.ja.md)

### 修正

#### ネットワーキング

v0.1.0 では、プロトコルごとに個別のスレッドと Tokio ランタイムを起動していたため、リソースの重複消費が発生していました。今回の更新では共有ランタイムハンドルを使用するよう変更し、システム全体のリソース効率が大幅に向上しました。

#### 組み込みプラグイン

- 各プラグインのスケジュールを整理・改善しました。
- システム登録時の実行順序を保証するため、`.chain` を実装しました。

---

## [0.1.0] - 2026-04-04

### 初回リリース

Ecson の最初の公開リリースです。
`tokio` の非同期 I/O と `bevy_ecs` のデータ指向設計を組み合わせた、ECS 駆動のステートフル双方向サーバーフレームワークの実験的実装です。

### 追加

#### コアフレームワーク

- `EcsonApp`: アプリケーションのエントリーポイント。ECS World とスケジューラーを統合します。
- ECS スケジューラーによるロックフリーの並列システム実行。
- `Update` および `FixedUpdate` スケジュールのサポート。
- `ecson::prelude::*` による公開 API の一括インポート。

#### ネットワーキング

- `EcsonWebSocketPlugin`: WebSocket (WS) サーバー（開発・本番共用の汎用サーバー）。
- `EcsonWebSocketTlsPlugin`: WebSocket over TLS (WSS) サーバー（PEM 証明書が必要）。
- `EcsonWebSocketTlsDevPlugin`: 自己署名証明書を自動生成する開発用 WSS サーバー。
- `EcsonWebTransportDevPlugin`: WebTransport (HTTP/3 / QUIC) 開発用サーバー（低遅延データグラム通信）。
- `MessageReceived`・`SendMessage`・`UserDisconnected` を活用した ECS ベースのネットワーキングイベント API。

#### 組み込みプラグイン

- `HeartbeatPlugin`: Ping/Pong による死活監視と自動切断（`ClientTimedOutEvent`）。
- `RateLimitPlugin`: メッセージ送信レート制限（Drop / Throttle / Disconnect）。
- `PresencePlugin`: クライアントのプレゼンス状態管理（Online / Away / Busy）。
- `SnapshotPlugin`: ECS コンポーネントの定期スナップショット送信（差分モード対応）。
- `LobbyPlugin`: マッチメイキング用ロビー機能（`LobbyReadyEvent`）。
- `ChatCorePlugin`: ニックネーム設定とグローバルブロードキャスト。
- `ChatRoomPlugin`: ルームへの参加・退出およびルーム一覧機能（`ChatCorePlugin` と併用）。
- `ChatFullPlugin`: チャット関連機能をすべて統合したオールインワンプラグイン。
- `Spatial2DPlugin`: 2D 空間の位置管理および AOI（Area of Interest）通知。
- `Spatial3DFlatPlugin`: 地上ベースの 3D 空間向け AOI 通知（RPG・MOBA など）。
- `Spatial3DPlugin`: フル 3D 空間向け AOI 通知（宇宙・フライトシミュレーターなど）。

#### サンプル

- `examples/echo.rs`: 接続クライアントへのエコーサーバー。
- `examples/broadcast_chat.rs`: 全クライアントへのブロードキャストチャット。
- `examples/room_chat.rs`: ルームベースのチャット。

#### ドキュメント

- [公式ドキュメント](https://ecson.netlify.app/)

---

> ⚠️ **このバージョンは実験的なリリースです。** 本番環境での使用は推奨しません。API は予告なく変更される場合があります。

[0.1.1]: https://github.com/hino-rs/ecson/releases/tag/v0.1.1
[0.1.0]: https://github.com/hino-rs/ecson/releases/tag/v0.1.0