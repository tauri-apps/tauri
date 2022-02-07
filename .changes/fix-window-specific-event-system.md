---
"tauri": patch
---

The `tauri::Window#emit` functiow now correctly sends the event to all windows that has a registered listener.
**Breaking change:** `Window#emit_and_trigger` and `Window#emit` now requires the payload to be cloneable.
