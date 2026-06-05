---
"tauri-utils": "patch:bug"
---

Fix panic when converting `Number::Float(NaN)` or `Number::Float(±Infinity)` to `serde_json::Value`.

`From<Value> for serde_json::Value` was calling `serde_json::Number::from_f64(f).unwrap()`, which panics for non-finite floats because the JSON specification (RFC 8259) does not permit them. Non-finite values now map to `serde_json::Value::Null` instead of panicking.
