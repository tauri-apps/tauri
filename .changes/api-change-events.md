---
"tauri": patch
---

- `tauri::event::Event` and `tauri::event::EventHandler` are now exported as `tauri::Event` and `tauri::EventHandler`
- **breaking**: The old `tauri::app::Event` has been renamed to `tauri::app::RunEvent` and is available as `tauri::RunEvent`
