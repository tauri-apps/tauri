---
'tauri': 'patch:breaking'
---

Removed `App/AppHandle::tray` and `App/AppHandle::remove_tray`, use `App/AppHandle::tray_by_id` and `App/AppHandle::remove_tray_by_id` instead. If these APIs were used to access tray icon configured in `tauri.conf.json`, you can use `App/AppHandle::tray_by_id` with ID `main` or the configured value.
