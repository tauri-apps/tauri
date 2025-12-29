---
tauri-runtime-wry: patch:bug
---

On Linux, keep the WebContext alive to prevent zombie WebKit processes after repeatedly closing all windows and re-opening them.
