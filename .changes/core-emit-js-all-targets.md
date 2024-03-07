---
'tauri': 'patch:bug'
---

Fix `emit` and `emit_to` (when used with `EventTarget::Any`) alawys skipping the webview listeners.
