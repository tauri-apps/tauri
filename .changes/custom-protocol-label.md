---
"tauri": "patch:breaking"
---

Changed uri scheme protocol handler to take `UriSchemeContext` as first argument instead of `AppHandle`. `UriSchemeContext` can be used to access an app handle or the webview label that made the request. The following methods are affected:
- `tauri::Builder::register_uri_scheme_protocol`
- `tauri::Builder::register_asynchronous_uri_scheme_protocol`
- `tauri::plugin::Builder::register_uri_scheme_protocol`
- `tauri::plugin::Builder::register_asynchronous_uri_scheme_protocol`
