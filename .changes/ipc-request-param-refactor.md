---
"tauri-runtime": patch:breaking
"tauri-runtime-wry": patch:breaking
---

The IPC handler closure now receives a `http::Request` instead of a String representing the request body.
