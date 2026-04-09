# Ecson v0.2.4

[English](https://github.com/hino-rs/ecson/blob/main/README.md)

> Rust向けの、ECS駆動なステートフル双方向サーバーフレームワーク

**Ecson（エクソン）** は、リアルタイム性とステート管理が重要なアプリケーションのために構築された実験的なWebサーバーフレームワークです。`tokio` の非同期I/O処理能力と `bevy_ecs` の超高速なデータ志向設計を統合することで、`Arc<Mutex<T>>` のような同期プリミティブを一切使用することなく、マルチプレイヤーゲームのバックエンド・リアルタイムコラボレーションツール・空間シミュレーションなどを構築できます。

> ⚠️ **本プロジェクトは実験段階です。** 本番環境への使用は推奨しません。APIは予告なく変更される場合があります。

---

## なぜECSをWebサーバーに？

従来の非同期Webフレームワーク（axum、Actix-web）は、ステートレスなCRUD APIの構築には最適です。しかし、数千のWebSocket接続が同時に・継続的に状態を共有し合うリアルタイムアプリケーションを構築しようとすると、すぐに**グローバル共有状態**という壁にぶつかります。

```rust
// よくある「つらい」パターン
let state = Arc::new(Mutex::new(AppState::new()));

// ロックの競合、デッドロック、スレッドのブロック...
let mut s = state.lock().await;
```

Ecsonはこの問題に対し、**ECS（Entity Component System）** というパラダイムで向き合います。

| 従来の概念 | Ecsonでの扱い |
|:---|:---|
| クライアント接続 | Entity（エンティティ） |
| ユーザーの状態・属性 | Component（コンポーネント） |
| ビジネスロジック | System（システム） |

各接続はグローバルなECSワールド内で1つのエンティティとして生成されます。ECSスケジューラはデータの競合が発生しないシステムを自動的に並行実行します。**ロックは一切不要です。**

---

## 向いているケース・向いていないケース

**向いているケース**

- マルチプレイヤーゲームのバックエンド
- ライブホワイトボードやリアルタイムコラボレーションツール
- メタバース・空間シミュレーション
- 在席管理を伴うチャットシステム

**向いていないケース**

- REST APIやシンプルなHTTPサーバー
- ステートレスなCRUDアプリケーション

---

## クイックスタート

```bash
cargo new my-ecson-server
cd my-ecson-server
cargo add ecson
```

`src/main.rs` を編集します。

```rust
use ecson::prelude::*;

fn echo_system(
    mut ev_recv: MessageReader<MessageReceived>,
    mut ev_send: MessageWriter<SendMessage>,
) {
    for msg in ev_recv.read() {
        ev_send.write(SendMessage {
            target: msg.entity,
            payload: msg.payload.clone(),
        });
    }
}

fn main() {
    EcsonApp::new()
        .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
        .add_systems(Update, echo_system)
        .run();
}
```

```bash
cargo run
# EcsonApp Started🚀
```

ブラウザのコンソールで動作確認できます。

```javascript
const ws = new WebSocket("ws://127.0.0.1:8080");
ws.onmessage = (e) => console.log("Received:", e.data);
ws.onopen = () => ws.send("Hello, Ecson!");
// → Received: Hello, Ecson!
```

---

## 組み込みプラグインでさらに簡単に

組み込みプラグインを使えば、複雑な機能もすぐに追加できます。例えばルーム付きチャットサーバーはたった数行で完成します。

```rust
use ecson::prelude::*;
use ecson::plugins::chat::ChatFullPlugin;

fn main() {
    EcsonApp::new()
        .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run();
}
```

主な組み込みプラグインは以下の通りです。

| プラグイン | 機能 |
|:---|:---|
| `ChatFullPlugin` | `/nick`・`/join`・`/list` など、ルーム付きチャット機能 |
| `HeartbeatPlugin` | 定期的なPingで応答のないクライアントを自動切断 |
| `LobbyPlugin` | マッチメイキング向けのロビー管理 |
| `PresencePlugin` | クライアントの在席状態（Online / Away / Busy）管理 |
| `RateLimitPlugin` | メッセージ送信頻度の制限 |
| `SnapshotPlugin` | エンティティ状態の定期スナップショット送信 |
| `Spatial2DPlugin` など | 2D/3Dの位置管理とAOI（Area of Interest）通知 |

---

## 対応プロトコル

| プラグイン | プロトコル | 用途 |
|:---|:---|:---|
| `EcsonWebSocketPlugin` | WS | 開発・本番（汎用） |
| `EcsonWebSocketTlsPlugin` | WSS | 本番環境（TLS） |
| `EcsonWebSocketTlsDevPlugin` | WSS（自己署名） | 開発・テスト用 |
| `EcsonWebTransportDevPlugin` | WebTransport | 低レイテンシ通信・ゲーム向け |

---

## サンプル

```bash
cargo run --example echo
cargo run --example broadcast_chat
cargo run --example room_chat
cargo run --example spatial_2d
```

---

## ログ出力

Ecsonは `tracing` クレートを使用しています。ログを表示するには `tracing-subscriber` などをアプリ側で初期化してください。

```rust
fn main() {
    tracing_subscriber::fmt::init();
    EcsonApp::new()
        // ...
        .run();
}
```

---

## ワークスペース構成

```
ecson/
├── Cargo.toml
├── crates/
│   ├── ecson_core/     ← EcsonApp, Plugin, スケジュールラベル
│   ├── ecson_ecs/      ← ECS型, 組み込みプラグイン
│   ├── ecson_network/  ← WebSocket, WebTransport, TLS
│   └── ecson_macros/   ← deriveマクロ
└── ecson/              ← 公開ファサードクレート
    └── examples/
```

ユーザーが依存するクレートは `ecson` のみです。内部クレートは直接使用することは現状想定していません。

---

## 技術スタック

- [bevy_ecs](https://crates.io/crates/bevy_ecs)
- [tokio](https://crates.io/crates/tokio) / tokio-tungstenite / tokio-rustls / tokio-util
- [wtransport](https://crates.io/crates/wtransport)
- [tracing](https://crates.io/crates/tracing) / tracing-subscriber
- [futures-util](https://crates.io/crates/futures-util)
- [rcgen](https://crates.io/crates/rcgen) / rustls / rustls-pemfile

---

## ドキュメント

- [https://ecson.netlify.app/](https://ecson.netlify.app/)

## 記事

- [Rust向けのECS駆動な双方向通信サーバーフレームワークをリリースしました(Ecson)](https://zenn.dev/hino_rs/articles/73f711641e34df)

---

## コントリビューション

バグ修正・ドキュメント改善・サンプル追加など、あらゆる貢献を歓迎しています。大きな機能追加や設計変更を伴うものは、事前にIssueで議論してください。詳細は [CONTRIBUTING.md](./CONTRIBUTING.md) を参照してください。

**必要な環境:** Rust 1.85以上（`edition = "2024"` を使用）

---

## ライセンス

[MIT License](./LICENSE)

---

## 謝辞

本プロジェクトは主に [Tokio](https://tokio.rs) と [Bevy ECS](https://bevyengine.org) という巨人の肩の上に立っています。また、開発にあたり以下のツールにお世話になりました。

- [GitHub](https://github.com)
- [Gemini](https://gemini.google.com)
- [ChatGPT](https://chatgpt.com)
- [Claude](https://claude.ai)
- [PLaMo翻訳](https://app.translate.preferredai.jp/)
- [mdBook](https://github.com/rust-lang/mdBook)