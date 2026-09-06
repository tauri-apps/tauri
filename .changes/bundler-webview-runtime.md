---
"tauri-bundler": minor:feat
---

Added `BundleSettings::webview_runtime` to tell the bundler which webview runtime the application uses, so it only ships what that runtime needs:

- `WebviewRuntime::Wry` (the default): the AppImage ships the webkit2gtk helper processes, and the NSIS and WiX installers include the WebView2 installation step (`webviewInstallMode`, `minimumWebview2Version` and the `WebView2Loader.dll` resource).
- `WebviewRuntime::Cef { distribution: Some(path) }`: the CEF distribution at `path` is copied into the bundle.
- `WebviewRuntime::Cef { distribution: None }`: the app loads CEF from a shared runtime outside its bundle, so no CEF files are shipped (the macOS helper apps are still created).
- `WebviewRuntime::Other`: nothing runtime-specific is shipped.

The `BundleSettings::cef_path` and `BundleSettings::cef_shared_runtime` fields were removed in favor of the `Cef` variant.
