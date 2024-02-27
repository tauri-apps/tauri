---
"tauri": patch:bug
---

Adds the `unlisten_all_js` method to struct `Webview` and calls it in on_page_load_handler when the corresponding webview window is refreshed.
This will only clean up all `js_listeners` for the window that was refreshed, and will not affect other windows.
