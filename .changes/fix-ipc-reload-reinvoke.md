---
"tauri": patch:bug
---

Fixes an issue where an in-flight asynchronous command was invoked a second time when the webview was reloaded (or navigated away) while the command was still running. The custom protocol `fetch` request cancelled by the reload is no longer treated as a custom protocol failure, so it no longer falls back to the `postMessage` interface and re-sends the message.
