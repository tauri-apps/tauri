---
"tauri-runtime-wry": patch:bug
---

Revert webview's visibility doesn't change with the app window, the previous change causes flickering on show/restore, so revert it for now
