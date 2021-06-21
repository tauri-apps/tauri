---
"tauri": patch
---

**Breaking change**: The global shortcut API is now managed by `tao` so it cannot be accessed globally, the manager is now exposed on the `App` and `AppHandle` structs.
