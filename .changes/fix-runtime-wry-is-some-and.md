---
"tauri-runtime-wry": patch
---

Fix "expected 2 arguments, found 1" compilation error in `lib.rs` by replacing `is_some_and` with `map_or` for better compatibility.
