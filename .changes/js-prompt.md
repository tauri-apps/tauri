---
"tauri": patch
---

Use native dialog on `window.alert` and `window.confirm`.
Since every communication with the webview is asynchronous, the `window.confirm` returns a Promise resolving to a boolean value.
