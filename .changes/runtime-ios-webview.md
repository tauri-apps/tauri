---
"tauri-runtime": major:breaking
---

Added the required `WebviewDispatch::with_ios_webview` method on iOS, which gives access to the platform webview, plugin manager and view controller pointers through the new `webview::IosWebviewHandle`, so `tauri` no longer downcasts to the wry webview.
