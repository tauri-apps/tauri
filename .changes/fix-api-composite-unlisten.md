---
"@tauri-apps/api": patch:bug
---

Return an awaitable cleanup function from `Window.onDragDropEvent`, `Window.onFocusChanged` and `Webview.onDragDropEvent` so rejections from the inner unlisten calls can be handled instead of surfacing as unhandled promise rejections.
