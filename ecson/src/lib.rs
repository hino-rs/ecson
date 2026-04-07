//! # ecson
//!
//! ECS 駆動のステートフル双方向サーバーフレームワーク。
//!
//! 通常は [`prelude`] を glob import するだけで使い始められます。
//!
//! ```rust
//! use ecson::prelude::*;
//! ```
//!
//! ビルトインプラグインは [`plugins`] モジュールから利用できます。
//!
//! ```rust
//! use ecson::plugins::chat::ChatFullPlugin;
//! ```

// ── 内部クレートの再エクスポート ────────────────────────────────────────────
// `bevy_ecs` はユーザーが直接触る必要がある高度なユースケース（カスタム System など）
// のために公開するが、`#[doc(hidden)]` で通常ドキュメントには現れないようにする。
// 内部クレートは原則非公開とし、`prelude` / `plugins` 経由でのみ使う。
#[doc(hidden)]
pub use bevy_ecs;

/// ビルトインプラグイン群。
///
/// `use ecson::plugins::chat::ChatFullPlugin;` のようにアクセスします。
pub use ecson_ecs::plugins;

// ── prelude ─────────────────────────────────────────────────────────────────
/// `ecson::prelude::*` で一括 import できる公開 API セット。
///
/// ユーザーコードでは通常このモジュールだけを使います。
pub mod prelude {
    // --- ネットワーク / ECS メッセージング ---
    pub use bevy_ecs::event::Event;
    pub use bevy_ecs::message::{MessageReader, MessageWriter, Messages};
    pub use bevy_ecs::prelude::*;

    // --- アプリケーション & スケジュール ---
    pub use ecson_core::app::{EcsonApp, FixedUpdate, Shutdown, Startup, Update};

    // --- プラグイン基盤 ---
    pub use ecson_core::plugin::{Plugin, Plugins, PluginsState};

    // --- サーバー設定 ---
    pub use ecson_core::server_time_config::ServerTimeConfig;

    // --- ライフサイクル ---
    pub use ecson_core::app::ShutdownFlag;

    // --- ネットワーク型 ---
    pub use ecson_ecs::channels::{NetworkEvent, NetworkPayload};

    // --- ECS 層 ---
    pub use ecson_ecs::components::*;
    pub use ecson_ecs::events::*;
    pub use ecson_ecs::resources::*;
    pub use ecson_ecs::systems::*;

    // --- ネットワークプラグイン ---
    pub use ecson_network::plugin::*;
}