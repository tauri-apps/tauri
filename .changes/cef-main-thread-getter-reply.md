---
"tauri-runtime-cef": patch:bug
---

Return an error instead of deadlocking a window or webview getter called from a main-thread CEF callback outside winit dispatch. Inline replies and worker-thread calls keep their existing behavior. Applications should move blocking getters out of synchronous IPC handlers.
