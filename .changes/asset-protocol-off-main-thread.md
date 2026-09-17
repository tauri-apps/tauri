---
tauri: patch:bug
---

Read `asset://` files off the event loop thread, so a slow or unreachable path no longer freezes every window.
