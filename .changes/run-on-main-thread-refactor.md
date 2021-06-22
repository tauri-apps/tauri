---
"tauri-runtime-wry": patch
"tauri": patch
---

The `run_on_main_thread` API now uses WRY's UserEvent, so it wakes the event loop.
