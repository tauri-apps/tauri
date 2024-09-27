---
"tauri": "patch:breaking"
---

Changed uri scheme protocol handler function to take an additional argument of type `&str` which is the webview label that made the request:
- `tauri::Builder::register_uri_scheme_protocol`
- `tauri::Builder::register_asynchronous_uri_scheme_protocol`
- `tauri::plugin::Builder::register_uri_scheme_protocol`
- `tauri::plugin::Builder::register_asynchronous_uri_scheme_protocol`
