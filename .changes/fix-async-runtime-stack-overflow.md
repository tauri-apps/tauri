---
"tauri": patch
---

Fix stack overflow on Windows on commands by changing the implementation of the `async_runtime::spawn` method.
