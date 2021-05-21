---
"tauri-codegen": patch
"tauri-utils": patch
"tauri": patch
---

**Breaking:** The `assets` field on the `tauri::Context` struct is now a `Arc<impl Assets>`.
