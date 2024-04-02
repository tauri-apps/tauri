---
'tauri': 'patch:breaking'
---

`Manager::resources_table` is now scoped so each `App/AppHandle/Window/Webview/WebviewWindow` has its own resource collection.
