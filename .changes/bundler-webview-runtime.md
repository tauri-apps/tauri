---
"tauri-bundler": minor:feat
---

Added `BundleSettings::webview_runtime` (`WebviewRuntime::{Wry, Cef { distribution }, Other}`, defaults to `Wry`) so the bundler only ships what the webview runtime needs: the webkit2gtk helper processes in the AppImage, and the WebView2 installation step (`webviewInstallMode`, `minimumWebview2Version` and the `WebView2Loader.dll` resource) in the NSIS and WiX installers are only included for the wry runtime. It replaces the `cef_path` and `cef_shared_runtime` settings: `WebviewRuntime::Cef { distribution: Some(path) }` embeds the CEF distribution and `WebviewRuntime::Cef { distribution: None }` is an app on a shared CEF runtime.
