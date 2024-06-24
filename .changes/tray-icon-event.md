---
"tauri": "patch:breaking"
"@tauri-apps/api": "patch:breaking"
---

This release contains breaking changes to the tray event structure because of newly added events:
- Changed `TrayIconEvent` to be an enum instead of a struct.
- Added `MouseButtonState` and `MouseButton` enums.
- Removed `ClickType` enum and replaced it with `MouseButton` enum.
- Added `MouseButtonState` enum.
