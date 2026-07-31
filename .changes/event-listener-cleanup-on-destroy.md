---
"tauri": "patch:bug"
---

Remove Rust-side event listeners bound to a window or webview when that target is destroyed, so `listen`/`once` handlers registered on a `Window`, `Webview` or `WebviewWindow` no longer leak after it is closed.
