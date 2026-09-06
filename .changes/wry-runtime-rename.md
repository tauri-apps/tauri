---
"tauri-runtime-wry": major:breaking
---

The `Wry<T>` runtime type was renamed to `WryRuntime<T>` (defaulting to `tauri::EventLoopMessage`), and `Wry` is now the unit-like attributes type that selects the runtime, e.g. `tauri::Builder::default().runtime(tauri_runtime_wry::Wry::default())`. `WryHandle::plugin` now takes `&self`.
