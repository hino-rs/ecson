# プラグイン

2026/03/31現在提供されているプラグインは下記のとおりです。

`ecson::plugins::network`より、

- `EcsonWebSocketPlugin`
- `EcsonWebTransportPlugin`

`ecson::plugins::chat`より、

- `ChatCorePlugin`
- `ChatRoomPlugin`
- `ChatFullPlugin`

`ecson::plugins::heartbeat`より、

- `HeartbeatPlugin`

`ecson::plugins::presence`より、

- `PresencePlugin`

## chat系プラグインを使ってみよう

実は、前章で作ってきたようなサーバーはプラグインによって爆速で開発できます。

`ChatCorePlugin`はブロードキャストなど、`ChatRoomPlugin`はルーム関係を実装しています。`ChatFullPlugin`はそれらを総合しています。

では、ルーム付きチャットサーバーを作ってみましょう。

```Rust
use ecson::prelude::*;
use ecson::plugins::chat::ChatFullPlugin;

fn main() {
    EcsonApp::new()
        .add_plugins(EcsonWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run()
}
```

これだけで簡単なチャットサーバーが作れます。
