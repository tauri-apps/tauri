---
'tauri': 'patch:bug'
---

Fix `emit` and `emit_to` (when used with `EventTarget::Any`) always skipping the webview listeners.
