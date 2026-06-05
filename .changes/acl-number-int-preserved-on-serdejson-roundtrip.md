---
"tauri-utils": "patch:bug"
---

Fix `Number::Int` being silently coerced to `Number::Float` on `serde_json` round-trip.

`From<serde_json::Value> for Value` was checking `as_f64()` first, which succeeds for every integer that fits in an f64, so integer JSON numbers were always deserialized as `Number::Float`. The check order is now `as_i64()` → `as_u64()` (falling back to `Float` for values above `i64::MAX` to avoid silent wrapping) → `as_f64()`, matching serde_json's own visitor convention.
