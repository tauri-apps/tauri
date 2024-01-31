---
'tauri': 'patch:breaking'
---

Refactored the event system to better accommodate the new window types:

- Added `EventTarget` enum.
- `App/AppHandle/Window/Webview/WebviewWindow::emit` will now emit to all event listeners.
- `App/AppHandle/Window/Webview/WebviewWindow::emit_to` will emit to event targets that match the given label, see `EventTarget` enum.
- `App/AppHandle/Window/Webview/WebviewWindow::emit_filter` will emit to event targets based on a filter callback which now takes `&EventTarget` instead of `&Window`.
