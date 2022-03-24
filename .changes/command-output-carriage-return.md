---
"tauri": patch
---

The `tauri::api::process::Command` API now properly reads stdout and stderr messages that ends with a carriage return (`\r`) instead of just a newline (`\n`).
