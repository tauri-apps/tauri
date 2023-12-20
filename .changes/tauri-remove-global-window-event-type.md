---
'tauri': 'major:breaking'
---

Removed `GlobalWindowEvent` struct, and unpacked its field to be passed directly to `tauri::Builder::on_window_event`.
