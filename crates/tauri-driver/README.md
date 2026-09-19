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
- macOS: **scaffold only** — see [MACOS_DRIVER_DESIGN.md](./MACOS_DRIVER_DESIGN.md)
  and [#7068](https://github.com/tauri-apps/tauri/issues/7068). Running
  `tauri-driver` on macOS today exits with a pointer to that document; no
  WebDriver session will be created.

_note: macOS does not currently have a first-party WebDriver implementation
for `WKWebView`. The design doc walks through the candidate approaches
(`safaridriver`, `appium-mac2-driver`, a `wry`-side bridge) and what the
scaffold does and does not do._

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
[wry]: https://github.com/tauri-apps/wry
[Tauri]: https://github.com/tauri-apps/tauri
