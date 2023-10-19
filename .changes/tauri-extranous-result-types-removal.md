---
'tauri': 'major:breaking'
---

- Removed `tauri::path::Error` and added its variants to `tauri::Error`
- Removed `tauri::path::Result` and `tauri::plugin::Result` aliases, you should use `tauri::Result` or your own `Result` type.
