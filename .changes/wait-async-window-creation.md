---
"tauri-runtime-wry": patch:bug
---

`create_window` and `create_webview` should wait for the window creation to complete before returning even if it's off main thread, it should also return the error if it failed
