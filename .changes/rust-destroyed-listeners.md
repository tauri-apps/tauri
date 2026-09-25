---
"tauri": "patch:bug"
---

Fix Rust `tauri://destroyed` listeners registered on a `Window` or `WebviewWindow` never running, because the window's listeners were purged before the event was emitted.
