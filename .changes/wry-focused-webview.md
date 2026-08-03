---
tauri-runtime-wry: minor:breaking
---

`CreateWebviewOptions::focused_webview` now takes `Arc<Mutex<FocusState>>` instead of `Arc<Mutex<Option<String>>>`
