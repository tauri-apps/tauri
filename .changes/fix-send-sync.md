---
"tauri-runtime-wry": minor:breaking
---

`WindowsStore` and `DispatcherMainThreadContext` are no longer `Send` and `Sync`, and unsafe impl has been moved to `Context` directly.
