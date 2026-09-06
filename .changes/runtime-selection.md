---
"tauri": major:breaking
---

The webview runtime is now selected when building the app instead of through Cargo features of the `tauri` crate. Applications depend on the runtime crate directly and pass its attributes to `tauri::Builder::runtime`:

```rust
tauri::Builder::default()
  .runtime(tauri_runtime_wry::Wry::default()) // or `tauri_runtime_cef::Cef::default()`
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
```

- The `wry` and `cef` features were removed, along with the `x11`, `dbus` and `macos-proxy` features that were forwarded to the wry runtime.
- The `tauri::Wry`, `tauri::WryHandle`, `tauri::Cef`, `tauri::CefHandle`, `tauri::CefDevToolsProtocol`, `tauri::CefRuntimeAttributes`, `tauri::run_cef_helper_process` and `tauri::CEF_API_VERSION_LAST` items were removed.
- `tauri::Builder::default()` uses the new type-erased `tauri::DynRuntime`, which is also the default runtime type of `AppHandle`, `Window`, `Webview` and the other generic types, so they can still be used without naming the runtime. Building the app fails with `RuntimeNotConfigured` if no runtime was selected.
- Static dispatch remains available with `tauri::Builder::<tauri_runtime_wry::WryRuntime>::new()`, whose `runtime` method takes the attributes of that runtime. The `runtime_init_attrs` builder method was merged into `runtime`.
