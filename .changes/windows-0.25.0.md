---
"tauri": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
---

Update the `windows` crate to 0.25.0, which comes with pre-built libraries. WRY and Tao can both reference the same types directly from the `windows` crate instead of sharing bindings in `webview2-com-sys`.