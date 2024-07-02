---
"tauri": "patch:breaking"
---

Renamed `DragDropEvent` enum variants to better convey when they are triggered:

- `DragDropEvent::Dragged` -> `DragDropEvent::Enter`
- `DragDropEvent::DragOver` -> `DragDropEvent::Over`
- `DragDropEvent::Dropped` -> `DragDropEvent::Drop`
- `DragDropEvent::Cancelled` -> `DragDropEvent::Leave`

This also comes with a change in the events being emitted to JS and Rust event listeners:

- `tauri://drag` -> `tauri://drag-enter`
- `tauri://drop` -> `tauri://drag-drop`
- `tauri://drop-over` -> `tauri://drag-over`
- `tauri://drag-cancelled` -> `tauri://drag-leave`
