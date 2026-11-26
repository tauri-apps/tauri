---
'tauri': 'patch:bug'
---

A `WebviewWindow` keeps behaving as one after a child webview is added to it. `Manager::get_webview_window`, `Manager::webview_windows` and the `WebviewWindow` command argument resolve the window's own webview (the one that shares the window's label) and no longer fail, and the window's drag and drop events keep reaching listeners on that webview.
