---
"tauri": "patch:bug"
---

Fix `Listener::once` can be called multiple times if they trigger `emit`(s) inside the handler
