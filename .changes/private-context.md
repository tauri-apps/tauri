---
"tauri": patch
"tauri-runtime": minor
"tauri-runtime-wry": minor
---

**Breaking:** `Context` fields are now private, and is expected to be created through `Context::new(...)`.
All fields previously available through `Context` are now public methods.
