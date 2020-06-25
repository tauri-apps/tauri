---
"tauri-api": patch
---

Fixes the httpRequest headers usage. It now accepts Strings instead of serde_json::Value.
