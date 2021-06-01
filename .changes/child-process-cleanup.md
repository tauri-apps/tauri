---
"tauri": patch
---

Kill child processes spawned with `tauri::api::process::Command` on `tauri::App` drop. Can be skipped with `tauri::Builder#skip_cleanup_on_drop`.
