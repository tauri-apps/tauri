---
"tauri": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
---

**Breaking:** `Context` fields are now private, and is expected to be created through `Context::new(...)`.
All fields previously available through `Context` are now public methods.
