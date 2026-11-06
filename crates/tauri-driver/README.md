# `tauri-driver` _(pre-alpha)_

Cross-platform WebDriver server for Tauri applications.

This is a [WebDriver Intermediary Node] that wraps the native WebDriver server
for platforms that [Tauri] supports. Your WebDriver client will connect to the
running `tauri-driver` server, and `tauri-driver` will handle starting the
native WebDriver server for you behind the scenes. It requires two separate
ports to be used since two distinct [WebDriver Remote Ends] run.

You can configure the ports used with arguments when starting the binary:

- `--port` (default: `4444`)
- `--native-port` (default: `4445`)

Supported platforms:

- Linux via `WebKitWebDriver`
- Windows via [Microsoft Edge Driver]
- **[Todo]** macOS via [Appium Mac2 Driver] (probably)

_note: the (probably) items haven't been proof-of-concept'd yet, and if it is
not possible to use the listed native webdriver, then a custom implementation
will be used that wraps around [wry]._

## WebDriver BiDi

`tauri-driver` speaks classic WebDriver and does not proxy the [WebDriver BiDi] websocket.
The BiDi `webSocketUrl` capability (auto-injected by clients like WebdriverIO 9+) is stripped before forwarding the session to the native driver, so clients fall back to classic WebDriver automatically.
With WebdriverIO you can also opt out explicitly by setting `'wdio:enforceWebDriverClassic': true` in your capabilities.

## Installation

You can install tauri-driver using Cargo:

```sh
cargo install tauri-driver --locked
```

## Trying it out

Check out the documentation at https://tauri.app/develop/tests/webdriver/,
including a small example application with WebDriver tests.

[WebDriver Intermediary Node]: https://www.w3.org/TR/webdriver/#dfn-intermediary-nodes
[WebDriver Remote Ends]: https://www.w3.org/TR/webdriver/#dfn-remote-ends
[Microsoft Edge Driver]: https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
[Appium Mac2 Driver]: https://github.com/appium/appium-mac2-driver
[wry]: https://github.com/tauri-apps/wry
[Tauri]: https://github.com/tauri-apps/tauri
