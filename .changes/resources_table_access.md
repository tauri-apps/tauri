---
'tauri': 'patch:breaking'
---

Removed `Manager::resources_table`, use `App/AppHandle/Window/Webview/WebviewWindow::resources_table` instead which will give access to dedicated resources table that is unique to each type.
