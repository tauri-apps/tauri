---
"tauri": minor
---

Added `async` support to the Tauri Rust core on commit [#a169b67](https://github.com/tauri-apps/tauri/commit/a169b67ef0277b958bdac97e33c6e4c41b6844c3).
This is a breaking change:
- Change `.setup(|dispatcher, source| {` to `.setup(|dispatcher, _source| async move {`.
- Change `.invoke_handler(|dispatcher, arg| {` to `.invoke_handler(|dispatcher, arg| async move {`.
- Add `.await` after `tauri::execute_promise()` calls.
