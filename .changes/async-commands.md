---
"tauri": patch
"tauri-macros": patch
---

Only commands with a `async fn` are executed on a separate task. `#[command] fn command_name` runs on the main thread.
