---
'tauri': 'patch:bug'
---

`Manager::get_webview_window`, `Manager::webview_windows` and the `WebviewWindow` command argument now resolve a window's own webview (the one that shares the window's label) when the window also holds child webviews. They used to fail as soon as a child webview was added to a `WebviewWindow`.
