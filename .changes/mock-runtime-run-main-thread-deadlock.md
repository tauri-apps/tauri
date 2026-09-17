---
tauri: patch:bug
---

Fix `run_main_thread!` macro used by things like `MenuItem` and some other APIs deadlock when called on main thread with `MockRuntime`
