# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.1.0] - 2026-04-04

### Initial Release

The first public release of Ecson.
An experimental implementation of an ECS-driven stateful bidirectional server framework, combining the asynchronous I/O of `tokio` with the data-oriented design of `bevy_ecs`.

### Added

#### Core Framework

- `EcsonApp` — Application entry point. Integrates the ECS World and scheduler.
- Lock-free parallel system execution via the ECS scheduler.
- Support for `Update` and `FixedUpdate` schedules.
- One-stop import for public APIs via `ecson::prelude::*`.

#### Networking

- `EcsonWebSocketPlugin` — WebSocket (WS) server (general purpose for development and production).
- `EcsonWebSocketTlsPlugin` — WebSocket over TLS (WSS) server (requires PEM certificate).
- `EcsonWebSocketTlsDevPlugin` — Development WSS server with auto-generated self-signed certificates.
- `EcsonWebTransportDevPlugin` — WebTransport (HTTP/3 / QUIC) development server (low-latency datagram communication).
- ECS-based networking event API utilizing `MessageReceived`, `SendMessage`, and `UserDisconnected`.

#### Built-in Plugins

- `HeartbeatPlugin` — Liveness monitoring and automatic disconnection via Ping/Pong (`ClientTimedOutEvent`).
- `RateLimitPlugin` — Message transmission rate limiting (Drop / Throttle / Disconnect).
- `PresencePlugin` — Client presence state management (Online / Away / Busy).
- `SnapshotPlugin` — Periodic snapshot transmission of ECS components (supports differential mode).
- `LobbyPlugin` — Lobby functionality for matchmaking (`LobbyReadyEvent`).
- `ChatCorePlugin` — Nickname setup and global broadcasting.
- `ChatRoomPlugin` — Room join/leave and listing functionality (used alongside `ChatCorePlugin`).
- `ChatFullPlugin` — All-in-one plugin combining all Chat-related features.
- `Spatial2DPlugin` — 2D spatial position management and AOI (Area of Interest) notifications.
- `Spatial3DFlatPlugin` — AOI notifications for ground-based 3D spaces (e.g., RPGs, MOBAs).
- `Spatial3DPlugin` — AOI notifications for full 3D spaces (e.g., space or flight simulators).

#### Examples

- `examples/echo.rs` — Echo server for connected clients.
- `examples/broadcast_chat.rs` — Broadcast chat to all clients.
- `examples/room_chat.rs` — Room-based chat.

#### Documentation

- [Tutorial](https://hino-rs.github.io/ecson/)
- [ECS Guide](https://github.com/hino-rs/ecson/tree/main/docs/ECS.md)
- [CONTRIBUTING.md](./CONTRIBUTING.md)

---

> ⚠️ **This version is experimental.** It is not recommended for production use. APIs are subject to change without notice.

[0.1.0]: https://github.com/hino-rs/ecson/releases/tag/v0.1.0