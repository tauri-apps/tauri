---
"tauri": minor
---

Moving the webview implementation to [webview](https://github.com/webview/webview), with the [official Rust binding](https://github.com/webview/webview_rust).
This is a breaking change.
IE support has been dropped, so the `edge` object on `tauri.conf.json > tauri` no longer exists and you need to remove it.
`webview.handle()` has been replaced with `webview.as_mut()`.
