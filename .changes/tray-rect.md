---
'tauri': 'patch:breaking'
---

Refactored the tray icon event struct:

- Changed `TrayIconEvent.icon_rect` type to use the new `tauri::Rect` type.
- Removed `TrayIconEvent.x` and `TrayIconEvent.y` fields and combined them into `TrayIconEvent.position` field.
- Removed `tauri::tray::Rectangle` struct.
