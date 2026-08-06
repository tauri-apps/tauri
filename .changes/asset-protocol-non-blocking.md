---
tauri: patch:bug
---

Read `asset://` files on the blocking thread pool instead of on the thread the webview invokes the protocol handler on, so a slow or unreachable path no longer freezes the whole application.
