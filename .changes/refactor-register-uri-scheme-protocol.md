---
"tauri": patch
"tauri-runtime": patch
---

**Breaking change:** Removed `register_uri_scheme_protocol` from the `WebviewAttibutes` struct and renamed `register_global_uri_scheme_protocol` to `register_uri_scheme_protocol` on the `Builder` struct, which now takes a `Fn(&AppHandle, &http::Request) -> http::Response` closure.
