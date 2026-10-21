---
"tauri": patch:bug
---

`tauri` now holds the per-window/per-webview event listeners itself and feeds them from the run event loop, which fixes `Window::on_window_event` and `Webview::on_webview_event` missing events fired before the registration reached the event loop.
