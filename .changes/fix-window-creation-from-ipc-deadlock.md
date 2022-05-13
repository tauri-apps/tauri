---
"tauri-runtime-wry": patch
---

Use the event loop proxy to create a window so it doesn't deadlock on Windows.
