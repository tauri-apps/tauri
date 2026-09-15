---
"tauri": major:breaking
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
---

Removed the deprecated `App::run_iteration` and the `Runtime::run_iteration` trait method it relied on. Use `App::run_return` to regain control of the flow after the event loop exits.
