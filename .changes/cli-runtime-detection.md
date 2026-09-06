---
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

The CLI detects the webview runtime (wry, CEF or other) from the `tauri-runtime-wry` and `tauri-runtime-cef` dependencies of the app manifest instead of the removed `cef` feature of `tauri`:

- The webkit2gtk package dependencies of the Debian and RPM packages and the WebView2 installation step of the Windows installers are only added when the app uses wry.
- The CEF files, code signing entitlements and macOS dev flow are only used when the app uses CEF.
- Nothing runtime-specific is done for other runtimes.
- The app and plugin templates add the `tauri-runtime-wry` dependency and select it with `tauri::Builder::default().runtime(tauri_runtime_wry::Wry::default())`.
