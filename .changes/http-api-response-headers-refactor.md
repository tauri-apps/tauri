---
"tauri": patch
---

**Breaking change:** The `tauri::api::http::Response#headers` method now returns `&header::HeaderMap` instead of `&HashMap`.
