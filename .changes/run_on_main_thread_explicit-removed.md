---
"tauri": "major:breaking"
---

Removed `App/AppHandle/WebviewWindow/Window/Webview::run_on_main_thread` method, just import `tauri::Manager` trait and use the new `Manager::run_on_main_thread`.

