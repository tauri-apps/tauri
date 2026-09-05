---
"tauri": major:breaking
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
"tauri-macros": major:breaking
"tauri-build": major:breaking
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

The webview runtime is no longer selected through Cargo features of the `tauri` crate. The `wry` and `cef` features (and the `x11`, `dbus` and `macos-proxy` features that were forwarded to the wry runtime) were removed, along with the `tauri::Wry`, `tauri::WryHandle`, `tauri::Cef`, `tauri::CefHandle`, `tauri::CefDevToolsProtocol`, `tauri::CefRuntimeAttributes`, `tauri::run_cef_helper_process`, `tauri::CEF_API_VERSION_LAST`, `tauri::cef_entry_point` and `tauri::webview_version` items.

Applications now depend on the runtime crate directly and select it when building the app:

```rust
tauri::Builder::default()
  .runtime(tauri_runtime_wry::Wry) // or `tauri_runtime_cef::Cef::default()`
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
```

`tauri::Builder::default()` uses the new type-erased `tauri::DynRuntime`, which is also the default runtime type of `AppHandle`, `Window`, `Webview` and the other generic types, so they can still be used without naming the runtime. Static dispatch remains available with `tauri::Builder::<tauri_runtime_wry::WryRuntime>::new()`.

Runtime-specific APIs moved to extension traits in the runtime crates: `tauri_runtime_wry::{AppHandleWryExt, AppWryExt, WebviewWryExt, WebviewWindowBuilderWryExt, WebviewBuilderWryExt}` (e.g. `create_tao_window`, `wry_plugin`, `with_environment`, `with_related_view`, `with_webview_configuration`) and `tauri_runtime_cef::{WebviewCefExt, WebviewWindowBuilderCefExt, WebviewBuilderCefExt}` (e.g. `send_dev_tools_message`, `on_dev_tools_protocol`, `browser_runtime_style`). `tauri_runtime_cef::cef_entry_point` replaces `tauri::cef_entry_point`, `tauri_runtime_wry::Wry<T>` was renamed to `WryRuntime<T>` and `tauri_runtime_cef::RuntimeInitAttrs` to `Cef`.

The `devtools` and `macos-private-api` features should now be enabled on the runtime crate, which also enables them on `tauri`. `tauri-build` detects the CEF runtime through the `tauri-runtime-cef` dependency, and the CLI detects the runtime (wry, CEF or other) from the app manifest instead of the `cef` feature: the webkit2gtk package dependencies and the WebView2 installation step of the Windows installers are only added when `tauri-runtime-wry` is used, and nothing runtime-specific is done for other runtimes.
