---
"tauri": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
---

Add `on_app_ready` to the `Builder` with access to the `AppHandle`.
The callback is called once the event loop is ready.
