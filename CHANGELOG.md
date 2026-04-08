# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.2.2]

Add proc-macro wrappers for Component, Resource, and Message

---

## [0.2.1]

Tighten public API surface of the facade crate

### Added

#### Core Framework

- `Plugins` and `PluginsState` are now exported from `ecson::prelude`
- `Shutdown` and `ShutdownFlag` are now exported from `ecson::prelude`
  (they were added in v0.2.0 but missing from the prelude)

### Changed

#### Core Framework

- `ecson/src/lib.rs`: removed bare `pub use` of internal crates
  (`ecson_core`, `ecson_ecs`, `ecson_network`) to prevent implementation
  details from leaking into the public API
- `bevy_ecs` re-export is now marked `#[doc(hidden)]` — it remains
  accessible for advanced use cases but no longer appears in public docs
- Added crate-level doc comments for the docs.rs landing page

---

## [0.2.0] - 2026-04-06

Major lifecycle improvements

### Added

#### Core Framework

- `Shutdown` schedule that runs once when the server shuts down
- `ShutdownFlag` to determine when the lifecycle is ending

#### Built-in Plugins

- `Plugin::cleanup` for cleanup processing during shutdown
- Changed the signature of `Plugin::build` from `self` to `&self`
- Changed `EcsonApp` to store plugins as `Vec<Box<dyn Plugin>>`
- After the main loop ends and before the `Shutdown` schedule runs, `cleanup` is now called for all plugins

---

## [0.1.1] - 2026-04-06

### Added

#### Core Framework

Added new methods to `EcsonApp`:

- `startup`: Executes `Startup` once.
- `tick_once`: Executes `Update` and `FixedUpdate` once.
- `tick_n(n)`: Executes `Update` and `FixedUpdate` n times.

#### Documentation

- [CHANGELOG.ja.md](./CHANGELOG.ja.md)

### Fixed

#### Networking

In v0.1.0, spawning separate threads and Tokio runtimes for each protocol led to redundant resource consumption. This update switches to using shared runtime handles, making the system much more resource-efficient.

#### Built-in Plugins

- Refined the schedules for each plugin.
- Implemented `.chain` to guarantee the correct execution order during system registration.

---

## [0.1.0] - 2026-04-04

### Initial Release

The first public release of Ecson.
An experimental implementation of an ECS-driven stateful bidirectional server framework, combining the asynchronous I/O of `tokio` with the data-oriented design of `bevy_ecs`.

### Added

#### Core Framework

- `EcsonApp`: Application entry point. Integrates the ECS World and scheduler.
- Lock-free parallel system execution via the ECS scheduler.
- Support for `Update` and `FixedUpdate` schedules.
- One-stop import for public APIs via `ecson::prelude::*`.

#### Networking

- `EcsonWebSocketPlugin`: WebSocket (WS) server (general purpose for development and production).
- `EcsonWebSocketTlsPlugin`: WebSocket over TLS (WSS) server (requires PEM certificate).
- `EcsonWebSocketTlsDevPlugin`: Development WSS server with auto-generated self-signed certificates.
- `EcsonWebTransportDevPlugin`: WebTransport (HTTP/3 / QUIC) development server (low-latency datagram communication).
- ECS-based networking event API utilizing `MessageReceived`, `SendMessage`, and `UserDisconnected`.

#### Built-in Plugins

- `HeartbeatPlugin`: Liveness monitoring and automatic disconnection via Ping/Pong (`ClientTimedOutEvent`).
- `RateLimitPlugin`: Message transmission rate limiting (Drop / Throttle / Disconnect).
- `PresencePlugin`: Client presence state management (Online / Away / Busy).
- `SnapshotPlugin`: Periodic snapshot transmission of ECS components (supports differential mode).
- `LobbyPlugin`: Lobby functionality for matchmaking (`LobbyReadyEvent`).
- `ChatCorePlugin`: Nickname setup and global broadcasting.
- `ChatRoomPlugin`: Room join/leave and listing functionality (used alongside `ChatCorePlugin`).
- `ChatFullPlugin`: All-in-one plugin combining all Chat-related features.
- `Spatial2DPlugin`: 2D spatial position management and AOI (Area of Interest) notifications.
- `Spatial3DFlatPlugin`: AOI notifications for ground-based 3D spaces (e.g., RPGs, MOBAs).
- `Spatial3DPlugin`: AOI notifications for full 3D spaces (e.g., space or flight simulators).

#### Examples

- `examples/echo.rs`: Echo server for connected clients.
- `examples/broadcast_chat.rs`: Broadcast chat to all clients.
- `examples/room_chat.rs`: Room-based chat.

#### Documentation

- [Official Documentation](https://ecson.netlify.app/)

---

> ⚠️ **This version is experimental.** It is not recommended for production use. APIs are subject to change without notice.

[0.1.1]: https://github.com/hino-rs/ecson/releases/tag/v0.1.1
[0.1.0]: https://github.com/hino-rs/ecson/releases/tag/v0.1.0