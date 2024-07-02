---
"@tauri-apps/api": "patch:breaking"
---

Renamed drag and drop events in `TauriEvent` enum to better convey when they are triggered:

- `TauriEvent.DRAG` -> `TauriEvent.DRAG_ENTER`
- `TauriEvent.DROP_OVER` -> `TauriEvent.DRAG_OVER`
- `TauriEvent.DROP` -> `TauriEvent.DRAG_DROP`
- `TauriEvent.DROP_CANCELLED` -> `TauriEvent::DRAG_LEAVE`

Also the `type` field values in `Window/Webview/WebviewWindow.onDropEvent` and `DragDropEvent` have changed:

- `dragged` -> `enter`
- `dragOver` -> `over`
- `dropped` -> `drop`
- `cancelled` -> `leave`
