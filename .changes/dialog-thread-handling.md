---
"tauri": patch
---

Allow the `tauri::api::dialog` APIs to be executed on any secondary thread.
**Breaking change:** All dialog APIs now takes a closure instead of returning the response on the function call.
