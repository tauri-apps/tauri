---
"tauri": patch:breaking
---

The `allows` and `denies` methods from `ipc::ScopeValue`, `ipc::CommandScope` and `ipc::GlobalScope` now returns `&Vec<Arc<T>>` instead of `&Vec<T>`.
