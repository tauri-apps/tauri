---
"tauri": patch:bug
---

Move `PluginApi::register_ios_plugin` behind the `wry` Cargo feature as `Webview::with_webview` is only available when that feature is enabled.
