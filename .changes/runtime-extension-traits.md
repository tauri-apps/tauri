---
"tauri": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
---

Runtime-specific APIs moved from the `tauri` crate to extension traits in the runtime crates, which now depend on `tauri`:

- `tauri_runtime_wry::{AppHandleWryExt, AppWryExt, WebviewWryExt, WebviewWindowBuilderWryExt, WebviewBuilderWryExt}` provide `create_tao_window`, `send_tao_window_event`, `wry_plugin`, `with_wry_webview`, `with_environment`, `with_related_view` and `with_webview_configuration`.
- `tauri_runtime_cef::{WebviewCefExt, WebviewWindowBuilderCefExt, WebviewBuilderCefExt}` provide `send_dev_tools_message`, `on_dev_tools_protocol` and `browser_runtime_style`.
- The traits are implemented both for the concrete runtime and for `tauri::DynRuntime`, returning `tauri_runtime::Error::RuntimeTypeMismatch` when the app runs on a different runtime.
- The `tauri::tao` and `tauri::wry` re-exports were removed, use `tauri_runtime_wry::{tao, wry}`.
- `tauri::webview::PlatformWebview::downcast_ref` was added to reach the runtime's webview type from `with_webview`, whatever the runtime generic in use.
