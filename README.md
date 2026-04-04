# Ecson

[日本語](./README.ja.md)

> An ECS-driven, stateful, bidirectional server framework for Rust

**Ecson** is an experimental web server framework built for applications where real-time performance and persistent state management are critical. By combining `tokio`'s async I/O with the blazing-fast data-oriented design of `bevy_ecs`, you can build multiplayer game backends, real-time collaboration tools, and spatial simulations — without a single `Arc<Mutex<T>>`.

> ⚠️ **This project is experimental.** It is not recommended for production use. The API may change without notice.

---

## Why ECS for a Web Server?

Traditional async web frameworks (axum, Actix-web) excel at building stateless CRUD APIs. But when you need thousands of persistent WebSocket connections continuously sharing and mutating state, you quickly hit the wall of **global shared state**.

```rust
// The painful pattern you know too well
let state = Arc::new(Mutex::new(AppState::new()));

// Lock contention, deadlocks, thread starvation...
let mut s = state.lock().await;
```

Ecson tackles this problem with **ECS (Entity Component System)** — an architecture pattern battle-tested in game development, now applied to server-side programming.

| Traditional Concept | Ecson Equivalent |
|:---|:---|
| Client connection | Entity |
| User state / attributes | Component |
| Business logic | System |

Each connection is spawned as an entity in a global ECS world. The ECS scheduler automatically runs non-conflicting systems in parallel across threads. **No locks required.**

---

## Good Fits & Poor Fits

**Good fits**

- Multiplayer game backends
- Live whiteboards and real-time collaboration tools
- Metaverse and spatial simulations
- Chat systems with presence management

**Poor fits**

- REST APIs and simple HTTP servers
- Stateless CRUD applications

---

## Quick Start

```bash
cargo new my-ecson-server
cd my-ecson-server
cargo add ecson
```

Edit `src/main.rs`:

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

Try it in your browser console:

```javascript
const ws = new WebSocket("ws://127.0.0.1:8080");
ws.onmessage = (e) => console.log("Received:", e.data);
ws.onopen = () => ws.send("Hello, Ecson!");
// → Received: Hello, Ecson!
```

---

## Built-in Plugins

Ecson ships with a set of plug-and-play plugins for common use cases. A full-featured room-based chat server, for example, is just a few lines:

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

| Plugin | What it does |
|:---|:---|
| `ChatFullPlugin` | Room-based chat with `/nick`, `/join`, `/list` commands |
| `HeartbeatPlugin` | Periodic ping/pong with automatic timeout disconnection |
| `LobbyPlugin` | Matchmaking lobby management with `LobbyReadyEvent` |
| `PresencePlugin` | Client presence state (Online / Away / Busy) |
| `RateLimitPlugin` | Per-client message rate limiting with configurable actions |
| `SnapshotPlugin` | Periodic state snapshots broadcast to subscribed clients |
| `Spatial2DPlugin` / `Spatial3DPlugin` | Spatial position management with AOI (Area of Interest) notifications |

---

## Network Protocols

| Plugin | Protocol | Use case |
|:---|:---|:---|
| `EcsonWebSocketPlugin` | WS | General purpose, dev & production |
| `EcsonWebSocketTlsPlugin` | WSS | Production with TLS certificates |
| `EcsonWebSocketTlsDevPlugin` | WSS (self-signed) | Development / testing |
| `EcsonWebTransportDevPlugin` | WebTransport | Low-latency games, position sync |

---

## Examples

```bash
cargo run --example echo
cargo run --example broadcast_chat
cargo run --example room_chat
cargo run --example spatial_2d
```

Frontend test HTML files are available in `examples/frontend/`. Open them directly in your browser to interact with the running server.

---

## Logging

Ecson uses the `tracing` crate internally. Initialize a subscriber in your application to see logs:

```rust
fn main() {
    tracing_subscriber::fmt::init();
    EcsonApp::new()
        // ...
        .run();
}
```

---

## Workspace Layout

```
ecson/
├── Cargo.toml
├── crates/
│   ├── ecson_core/     ← EcsonApp, Plugin trait, schedule labels
│   ├── ecson_ecs/      ← ECS types, built-in plugins
│   ├── ecson_network/  ← WebSocket, WebTransport, TLS
│   └── ecson_macros/   ← derive macros (future use)
└── ecson/              ← Public facade crate (re-exports everything)
    └── examples/
```

Users should only depend on the `ecson` crate. Internal crates are not intended for direct use.

---

## Tech Stack

- [bevy_ecs](https://crates.io/crates/bevy_ecs)
- [tokio](https://crates.io/crates/tokio) / tokio-tungstenite / tokio-rustls / tokio-util
- [wtransport](https://crates.io/crates/wtransport)
- [tracing](https://crates.io/crates/tracing) / tracing-subscriber
- [futures-util](https://crates.io/crates/futures-util)
- [rcgen](https://crates.io/crates/rcgen) / rustls / rustls-pemfile

---

## Documentation

[https://ecson.netlify.app/](https://ecson.netlify.app/)

---

## Contributing

Bug fixes, documentation improvements, and new examples are all welcome. For large feature additions or breaking design changes, please open an Issue first to discuss. See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

**Requirements:** Rust 1.85 or later (`edition = "2024"`)

---

## License

[MIT License](./LICENSE)

---

## Acknowledgements

This project stands on the shoulders of [Tokio](https://tokio.rs) and [Bevy ECS](https://bevyengine.org). We're also grateful for the following tools used during development:

- [GitHub](https://github.com)
- [Gemini](https://gemini.google.com)
- [ChatGPT](https://chatgpt.com)
- [Claude](https://claude.ai)
- [PLaMo translate](https://app.translate.preferredai.jp/)
- [mdBook](https://github.com/rust-lang/mdBook)
