---
"tauri-runtime-wry": major:breaking
---

The `WebviewAttribute` enum was replaced by the `WryWebviewAttributes` struct, with the `environment` (Windows), `related_view` (Linux) and `webview_configuration` (macOS) fields. The `AsWryWebviewAttributes` trait gives the `WebviewWindowBuilderWryExt` and `WebviewBuilderWryExt` extension traits access to it on both `WryRuntime` and `tauri::DynRuntime`.
