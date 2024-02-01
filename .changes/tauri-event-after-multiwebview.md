---
'tauri': 'patch:breaking'
---

Refactored the event system to better accommodate the new window types:

- Added `EventTarget` enum.
- Added `App/AppHandle::listen`, `App/AppHandle::once` and `App/AppHandle::unlisten` to listen to events targeting `App/AppHandle`
- `App/AppHandle/Window/Webview/WebviewWindow::emit` will now emit to all event listeners.
- `App/AppHandle/Window/Webview/WebviewWindow::emit_to` will emit to event targets that match the given label, see `EventTarget` enum.
- `App/AppHandle/Window/Webview/WebviewWindow::emit_filter` will emit to event targets based on a filter callback which now takes `&EventTarget` instead of `&Window`.
- Renamed `Manager::listen_global` and `Manager::once_global` to `listen_any` and `once_any` respectively to be consistent with `EventTarget::Any`, it will now also listen to any event to any target (aka event sniffer).
