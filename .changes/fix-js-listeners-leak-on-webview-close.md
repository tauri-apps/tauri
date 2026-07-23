---
'tauri': 'patch:bug'
---

Remove a webview's JS event listeners from the backend `Listeners` map when that webview is destroyed. Previously the entries keyed by the source webview label lingered after the webview was dropped, so they could never be delivered and leaked until the app exited — forcing apps to manually `unlisten()` before closing a window.
