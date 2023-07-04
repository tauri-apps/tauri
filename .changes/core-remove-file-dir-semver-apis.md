---
'tauri': 'patch:breaking'
---

- Removed `tauri::api::file` and `tauri::api::dir` modules, use `std::fs` instead.
- Removed `tauri::api::version` module, use `semver` crate instead.
