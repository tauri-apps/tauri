---
"tauri": patch
---

Ease the requirements for plugin hooks. `setup` and `setup_with_config` can now be `FnOnce` and `on_webview_ready`, `on_event` and `on_page_load` can be `FnMut`.
