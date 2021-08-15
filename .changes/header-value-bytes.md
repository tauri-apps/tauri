---
"tauri": patch
---

Use `HeaderValue::from_bytes` instead of `HeaderValue::from_str` and `HeaderValue#to_bytes` instead of `HeaderValue#to_str` to improve compatibility.
