---
"tauri-runtime-wry": patch:bug
---

Use `Arc` instead of `Rc` on global shortcut and tray types to prevent crashes on macOS.
