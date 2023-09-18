---
"tauri": patch:breaking
---

Changed `Builder::register_uri_scheme_protocol` to return a `http::Response` instead of `Result<http::Response>`. To return an error response, manually create a response with status code >= 400.
