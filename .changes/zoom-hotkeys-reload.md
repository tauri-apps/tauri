---
"tauri": patch:bug
"tauri-runtime-wry": patch:bug
---

Keep zoom hotkeys coherent across page reloads and calls to `set_zoom` on Linux and macOS by tracking the zoom factor per webview rather than per document.
