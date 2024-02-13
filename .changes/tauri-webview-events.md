---
'tauri': 'patch:feat'
---

Add webview-specific events for multi-webview windows:

- Add `WebviewEvent` enum
- Add `RunEvent::WebviewEevnt` variant.
- Add `Builder::on_webview_event` and `Webview::on_webview_event` methods.
