---
"tauri-driver": patch:bug
---

Strip the WebDriver BiDi `webSocketUrl` capability from `alwaysMatch` and every `firstMatch` entry before forwarding a new session to the native driver.
Clients like WebdriverIO 9+ auto-inject it, but tauri-driver does not proxy the BiDi websocket and native drivers without BiDi support reject such sessions.
