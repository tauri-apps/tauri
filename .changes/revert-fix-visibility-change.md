---
"tauri-runtime-wry": patch:bug
---

Revert the [fix](https://github.com/tauri-apps/tauri/pull/9246) for webview's visibility doesn't change with the app window on Windows as it caused white flashes on show/restore.
