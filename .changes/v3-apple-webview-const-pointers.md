---
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
---

The macOS and iOS platform webview accessors (`Webview::inner`, `Webview::controller`, `Webview::ns_window`, `Webview::view_controller`) and the fields of `IosWebviewHandle` are now `*const c_void` instead of `*mut c_void`, since the pointers are borrowed from ObjC `Retained` handles and must not be mutated through.
