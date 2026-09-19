---
'tauri': 'minor:changes'
'tauri-runtime-wry': 'minor:changes'
---

Replace the per-method `WindowMessage` variants in `tauri-runtime-wry` with closures dispatched to the event-loop thread. Setters on a closed window now return `WindowNotFound`. `start_resize_dragging` returns `NotSupported` on macOS. `set_cursor_grab`, `set_cursor_position`, `set_ignore_cursor_events`, `start_dragging` and `start_resize_dragging` now propagate OS errors.
