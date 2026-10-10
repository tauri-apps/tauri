---
"@tauri-apps/api": patch:bug
"tauri": patch:bug
---

Keep delivering `Channel` messages after the `onmessage` handler throws. The error is rethrown asynchronously so it is still reported as uncaught, instead of leaving every later message queued.
