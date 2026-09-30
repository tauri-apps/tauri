---
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
---

`tauri migrate` now migrates Tauri v2 apps to v3 (v1 and v2 beta apps continue to v3 after their existing migration):

- The Tauri crates and official plugins are updated to v3 in `Cargo.toml` and `package.json`, and the app depends on `tauri-runtime-wry`, which receives the `devtools`, `unstable`, `tracing`, `x11`, `dbus` and `macos-proxy` features (including the ones enabled through the app's own `[features]`). The removed `wry`, `macos-private-api` and `objc-exception` features are dropped.
- The `app > macOSPrivateApi` option is removed and the NSIS `install-icon` key is renamed to `installer-icon` in the main and platform-specific configuration files, keeping their formatting and comments.
- The Rust sources select the runtime with `.runtime(tauri_runtime_wry::Wry::default())` on `tauri::Builder::default()`, use `tauri::DynRuntime` instead of `tauri::Wry`, and pick up the renamed `with_inner_blocking`, `initialization_script`, `run_invoke_handler`, `tauri_runtime_wry::{tao, wry, android_binding, webview_version}` APIs.
