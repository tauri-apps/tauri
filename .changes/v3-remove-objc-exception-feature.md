---
"tauri": major:breaking
"tauri-runtime-wry": major:breaking
---

Removed the `objc-exception` Cargo feature from `tauri` and `tauri-runtime-wry`. It has been a no-op since 2.3.0 because wry no longer exposes it.
