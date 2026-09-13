---
'tauri-driver': 'patch:bug'
---

Strip the WebDriver BiDi `webSocketUrl` capability before forwarding a new session to the native driver. WebdriverIO 9+ auto-injects `webSocketUrl: true` to negotiate BiDi, but tauri-driver does not proxy the BiDi websocket and pre-2.46 WebKitGTK (shipped on current Ubuntu LTS) rejects sessions that request it, so the session failed to start. The capability is removed from both `alwaysMatch` and every `firstMatch` entry; BiDi is additive, so clients fall back to classic WebDriver.
