# Fluxion🌀

> A hyper-performant, ECS-driven, stateful WebSocket framework for Rust.

Fluxion is an experimental, highly opinionated Web Framework built from the ground up for **real-time, state-heavy applications**. By combining the asynchronous I/O power of `tokio` with the blazingly fast Data-Oriented Design of `bevy_ecs`, Fluxion allows you to build multiplayer backends, real-time collaboration tools, and spatial simulations—without ever writing `Arc<Mutex<T>>`.

## 💡 The Philosophy: Why ECS for Web?

Traditional async web frameworks (like Axum or Actix) are incredible for stateless CRUD APIs. But the moment you build a complex real-time server (e.g., a 2D metaverse, a live whiteboard, or a multiplayer game), you hit a wall: **Global Shared State**. Managing locking mechanisms across thousands of persistent WebSocket connections leads to deadlocks, thread blocking, and spaghetti code.

Fluxion solves this by shifting the paradigm:
1. **Connections are Entities:** Every WebSocket connection is spawned as an Entity in a global ECS World.
2. **Data is Components:** User IDs, coordinates, room names, and health points are just Components attached to these Entities.
3. **Logic is Systems:** You write pure functions (Systems) that query only the data they need. The ECS scheduler automatically runs non-conflicting systems in parallel across multiple threads. **Zero locks required.**

## Status

⚠️This is an experimental project.⚠️

- Not production-ready
- TLS not implemented yet
- API may change frequently

## 📄License

MIT License

## Build with

- bevy_ecs
- futures-util
- tokio
- tokio-tungstenite

## Examples

1. 01_hello_echo.rs:
    `cargo run --example 01_hello_echo`
    A server that returns received messages to their sender
2. 02_broadcast_chat.rs:
    `cargo run --example 02_broadcast_chat`
    A broadcast server that sends received messages to all connected users

## Acknowledgements

This project is built on top of amazing Rust ecosystem libraries:

- bevy_ecs — for its powerful and flexible ECS design
- tokio — for the asynchronous runtime foundation
- tokio-tungstenite — for WebSocket support

Huge thanks to the maintainers and contributors of these projects.