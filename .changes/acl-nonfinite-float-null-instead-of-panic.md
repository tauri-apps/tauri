---
"tauri-utils": "patch:bug"
---

Improve error handling for non-finite floats (`NaN`, `±Infinity`) in ACL values. TOML config parsing now rejects these values early with a clear message, and the JSON conversion panics with a descriptive error instead of a bare `unwrap()` failure.
