---
"tauri": patch
---

Allow disabling the webview file drop handler (required to use drag and drop on the frontend on Windows) using the `tauri.conf.json > tauri > windows > fileDropEnabled` flag or the `WebviewAttributes#disable_file_drop_handler` method.
