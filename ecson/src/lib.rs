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

/// ビルトインプラグイン群。
///
/// `use ecson::plugins::chat::ChatFullPlugin;` のようにアクセスします。
pub use ecson_ecs::plugins;

// ── prelude ─────────────────────────────────────────────────────────────────
/// `ecson::prelude::*` で一括 import できる公開 API セット。
///
/// ユーザーコードでは通常このモジュールだけを使います。
pub mod prelude {
    // --- bevy_ecsラッパー ---
    pub use ecson_ecs::types::*;

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

    // --- マクロ ---
    pub use ecson_macros::*;
}