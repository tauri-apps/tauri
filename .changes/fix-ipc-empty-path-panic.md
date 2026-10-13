---
"tauri": patch:bug
---

Fix a panic in the IPC and asset protocol handlers when the request URI has an empty path (for example `ipc:foo`).
