# プラグイン

2026/03/31現在提供されているプラグインは下記のとおりです。

`fluxion::plugins::network`より、

- `FluxionWebSocoketPlugin`
- `FluxionWebTransportPlugin`

`fluxion::plugins::chat`より、

- `ChatCorePlugin`
- `ChatRoomPlugin`
- `ChatFullPlugin`

## chat系プラグインを使ってみよう

実は、前章で作ってきたようなサーバーはプラグインによって爆速で開発できます。

`ChatCorePlugin`はブロードキャストなど、`ChatRoomPlugin`はルーム関係を実装しています。`ChatFullPlugin`はそれらを総合しています。

では、ルーム付きチャットサーバーを作ってみましょう。

```Rust
use fluxion::prelude::*;
use fluxion::plugins::chat::ChatFullPlugin;

fn main() {
    FluxionApp::new()
        .add_plugins(FluxionWebSocketPlugin::new("127.0.0.1:8080"))
        .add_plugins(ChatFullPlugin)
        .run()
}
```

これだけで簡単なチャットサーバーが作れます。
