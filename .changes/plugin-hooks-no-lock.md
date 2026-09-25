---
"tauri": "major:breaking"
---

Plugin hooks no longer run with the plugin store locked, so a plugin can add or remove plugins or exit the app from any of its callbacks without deadlocking. As a result, `Plugin` now requires `Sync`, and `window_created`, `webview_created`, `on_navigation`, `on_page_load`, `on_event` and `run_invoke_handler` take `&self` instead of `&mut self`, and may be called concurrently. `initialize` still takes `&mut self`.
