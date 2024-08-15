---
"tauri": patch:breaking
---

Added a dedicated type for IPC response body `InvokeResponseBody` for performance reasons.
This is only a breaking change if you are directly using types from `tauri::ipc`.
