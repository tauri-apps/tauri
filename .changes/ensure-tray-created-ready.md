---
"tauri-runtime-wry": patch:bug
---

Ensure system tray is created when the event loop is ready. Menu item modifications are not applied unless it is ready.
