---
"tauri": "major:breaking"
---

`plugin::Builder::on_page_load`, `on_window_ready`, `on_webview_ready` and `on_event` take a `Fn + Send + Sync` closure instead of `FnMut + Send`, and `on_navigation` now also requires `Sync`, because plugin hooks may now be called concurrently. Use interior mutability for state changed by these closures.
