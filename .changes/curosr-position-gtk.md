---
"tauri": "patch:bug"
"tauri-runtime-wry": "patch:bug"
---

Fix `App/AppHandle/Window/Webview/WebviewWindow::cursor_position` getter method failing on Linux with `GDK may only be used from the main thread`.
