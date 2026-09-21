---
tauri: patch:bug
---

Load `asset://` files asynchronously instead of on the event loop thread, so a slow or unreachable path no longer freezes every window.
