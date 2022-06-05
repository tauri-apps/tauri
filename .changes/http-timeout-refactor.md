---
"tauri": patch
---

**Breaking change:** The `api::http` timeouts are now represented as `std::time::Duration` instead of a `u64`.
