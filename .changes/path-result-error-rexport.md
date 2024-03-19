---
'tauri': 'major:breaking'
---

Removed `tauri::path::Result` and `tauri::path::Error` which were merely an unintentional re-export of `tauri::Result` and `tauri::Error` so use those instead.
