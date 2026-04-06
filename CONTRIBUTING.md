# Ecson コントリビューションガイド

Ecsonへの貢献に興味を持っていただきありがとうございます。
このドキュメントでは、開発環境のセットアップからPRの送り方まで説明します。

## 目次

- [開発環境のセットアップ](#開発環境のセットアップ)
- [ワークスペース構成](#ワークスペース構成)
- [クレートの役割と依存関係](#クレートの役割と依存関係)
- [ビルドとチェック](#ビルドとチェック)
- [サンプルの実行](#サンプルの実行)
- [どのクレートに追加するか](#どのクレートに追加するか)
- [コーディング規約](#コーディング規約)
- [ブランチとPRの運用](#ブランチとPRの運用)

---

## 開発環境のセットアップ

### 必要なもの

- Rust 1.85以上（`edition = "2024"` を使用しているため）
- Git

### セットアップ手順

```bash
git clone https://github.com/hino-rs/ecson.git
cd ecson
cargo build
```

---

## ワークスペース構成

このリポジトリはCargoワークスペースです。

```
ecson/
├── Cargo.toml               ← ワークスペース定義
├── crates/
│   ├── ecson_core/          ← EcsonApp, Plugin, スケジュールラベル
│   ├── ecson_ecs/           ← ECS型, チャンネル型, 組み込みプラグイン
│   ├── ecson_network/       ← WebSocket, WebTransport, TLS
│   └── ecson_macros/        ← deriveマクロ（将来用）
└── ecson/                   ← 公開ファサードクレート（pub use で再エクスポート）
    ├── src/lib.rs
    └── examples/            ← echo.rs, broadcast_chat.rs など
```

---

## クレートの役割と依存関係

### 依存グラフ

```
ecson_macros  （依存なし）
    │
ecson_core    (bevy_ecs, tracing)
    │
ecson_ecs     (ecson_core, bevy_ecs, tokio)
    │
ecson_network (ecson_ecs, ecson_core, tokio + ネットワーク系クレート)
    │
ecson         (上4クレートすべてを pub use)
```

**循環依存は禁止です。** グラフの上位クレートが下位クレートに依存する変更はできません。

### 各クレートの責務

| クレート | 主な内容 |
|---|---|
| `ecson_core` | `EcsonApp`、スケジュールラベル（`Update`/`FixedUpdate`/`Startup`）、`Plugin` トレイト、`ServerTimeConfig` |
| `ecson_ecs` | `NetworkPayload`/`NetworkEvent`（チャンネル型）、ECSコンポーネント・イベント・リソース・システム、組み込みプラグイン（chat, heartbeat, lobby, presence, rate_limit, snapshot, spatial） |
| `ecson_network` | WebSocket/WebTransport/WSS のサーバー実装、TLS ユーティリティ、`EcsonWebSocketPlugin` などのネットワーク系プラグイン |
| `ecson_macros` | derive マクロ（現在未実装） |
| `ecson` | 上記すべての再エクスポート、examples |

---

## ビルドとチェック

```bash
# ワークスペース全体をチェック
cargo check

# examples も含めてチェック
cargo check --examples

# 特定のクレートだけチェック
cargo check -p ecson_ecs

# リリースビルド
cargo build --release
```

### clippy と fmt

PRを送る前に以下を通してください。

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

---

## サンプルの実行

```bash
cargo run --example echo
cargo run --example broadcast_chat
cargo run --example room_chat
cargo run --example spatial_2d
```

フロントエンドのテスト用HTMLは `ecson/examples/frontend/` にあります。
ブラウザで直接開いて動作確認できます。

---

## どのクレートに追加するか

変更内容に応じて、追加先のクレートが変わります。

### 新しいプラグインを追加する

ECS層のみで完結するプラグイン（`bevy_ecs` のコンポーネント・システムだけで構成される）は `ecson_ecs/src/plugins/` に追加します。

```
ecson_ecs/src/plugins/
├── mod.rs         ← ここに `pub mod your_plugin;` を追加
└── your_plugin/
    ├── mod.rs     ← Plugin 実装, 公開型
    └── systems.rs ← システム実装
```

`ecson_ecs/src/plugins/mod.rs` に `pub mod your_plugin;` を追記するのを忘れずに。

ネットワークトランスポートが必要なプラグイン（Tokioランタイムを起動するもの）は `ecson_network/src/plugin.rs` に追加します。

### 新しいECS型（コンポーネント・イベント・リソース）を追加する

- コンポーネント → `ecson_ecs/src/components.rs`
- イベント（Message/Event） → `ecson_ecs/src/events.rs`
- リソース → `ecson_ecs/src/resources.rs`

追加した型は `ecson_ecs/src/lib.rs` の `prelude` から再エクスポートされているか確認してください。

### `EcsonApp` のAPIを変更する

`ecson_core/src/app.rs` を変更します。
`ServerTimeConfig` の変更は `ecson_core/src/server_time_config.rs` です。

### ネットワーク実装を変更する

`ecson_network/src/` 以下の対象ファイルを変更します。
チャンネル型（`NetworkPayload`, `NetworkEvent`）は `ecson_ecs/src/channels.rs` にあります。
これらの型は ECS 層と network 層の両方で使われるため、変更時は両側の影響を確認してください。

### `ecson` の公開APIに変更を加える

`ecson/src/lib.rs` の `prelude` モジュールを変更します。
ここは `ecson::prelude::*` でユーザーに届く型のゲートです。
不用意に型を追加・削除しないよう注意してください。

---

## コーディング規約

- **エディション**: `2024`
- **フォーマット**: `cargo fmt` の出力に従う
- **lint**: `cargo clippy -- -D warnings` が通ること
- **コメント**: 公開APIには日本語でdocコメントを書く（`///`）
- **エラー処理**: `unwrap()` はプラグインの初期化（起動時の設定ミスは即パニックでよい）以外では避ける
- **ログ**: `println!` ではなく `tracing::info!` / `tracing::error!` を使う

---

## ブランチとPRの運用

### ブランチ命名

| 種類 | 命名例 |
|---|---|
| 機能追加 | `feat/add-auth-plugin` |
| バグ修正 | `fix/ws-disconnect-panic` |
| リファクタリング | `refactor/ecson-ecs-split` |
| ドキュメント | `docs/contributing-guide` |

### PRのチェックリスト

- [ ] `cargo check --examples` が通る
- [ ] `cargo fmt --all` 適用済み
- [ ] `cargo clippy --all-targets -- -D warnings` が通る
- [ ] 変更内容を説明するPR概要を書いた
- [ ] 破壊的変更がある場合はその旨を明記した

### 受け付けている貢献の種類

- バグ修正
- ドキュメントの改善
- 既存プラグインの不具合修正
- サンプルの追加・改善

大きな機能追加や設計変更を伴うものは、事前にIssueで議論してください。

## PRテンプレート

```md
## Overview

## Changes

## Breaking Changes

## Checklist

- [x] cargo check --examples passes
- [x] Applied cargo fmt --all
- [x] cargo clippy --all-targets -- -D warnings passes
- [x] Wrote a PR overview explaining the changes
- [x] Clearly stated if there are any breaking changes
```
