---
"tauri-runtime": major:breaking
"tauri": major:breaking
---

Added the required `RuntimeHandle::webview_version` method, exposed as `App::webview_version` and `AppHandle::webview_version`. The `tauri::webview_version` function was removed since the version depends on the runtime in use.
