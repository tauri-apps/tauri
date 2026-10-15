---
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
---

`tauri_runtime::Error::CreateWindow` now carries the underlying error (`CreateWindow(Box<dyn std::error::Error + Send + Sync>)`), like `CreateWebview`.
