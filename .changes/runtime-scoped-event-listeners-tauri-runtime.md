---
"tauri-runtime": major:breaking
---

Removed `WindowDispatch::on_window_event` and `WebviewDispatch::on_webview_event`, along with the `WindowEventId` and `WebviewEventId` types. Runtimes already report every window and webview event through `RunEvent::WindowEvent` / `RunEvent::WebviewEvent`, so the per-window listener registries each runtime kept were a duplicate of that stream.

**Migration:** custom runtimes should drop their `on_window_event`/`on_webview_event` implementations and their listener storage; emitting `RunEvent::WindowEvent` and `RunEvent::WebviewEvent` is now the only thing required to deliver events.
