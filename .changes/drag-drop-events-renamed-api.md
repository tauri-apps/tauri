---
"@tauri-apps/api": "patch:breaking"
---

Renamed drag and drop events in `TauriEvent` enum to better convey when they are triggered:

- `TauriEvent.DRAG` -> `TauriEvent.DRAG_ENTER`
- `TauriEvent.DROP` -> `TauriEvent.DRAG_DROP`
- `TauriEvent.DROP_OVER` -> `TauriEvent.DRAG_OVER`
- `TauriEvent.DROP_CANCELLED` -> `TauriEvent::DRAG_LEAVE`
