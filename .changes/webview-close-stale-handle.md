---
"tauri": patch:bug
---

Fixed `Webview::close` on a handle to an already closed webview tearing down the state and event listeners of a newer webview reusing its label.
