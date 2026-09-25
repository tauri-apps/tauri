---
"tauri": "major:breaking"
---

Plugin hooks no longer run with the plugin store locked, so a plugin can add or remove plugins or exit the app from any of its callbacks without deadlocking. This changes the plugin APIs:

- `Plugin` now requires `Sync`, and `window_created`, `webview_created`, `on_navigation`, `on_page_load`, `on_event` and `run_invoke_handler` take `&self` instead of `&mut self`, and may be called concurrently. `initialize` still takes `&mut self`.
- `plugin::Builder::on_page_load`, `on_window_ready`, `on_webview_ready` and `on_event` take a `Fn + Send + Sync` closure instead of `FnMut + Send`, and `on_navigation` now also requires `Sync`. Use interior mutability for state changed by these closures.
- A plugin is only added to the app once its `initialize` (or `setup`) returns, so its own hooks do not run during it. A removed plugin is dropped once its hooks that are already running return.
