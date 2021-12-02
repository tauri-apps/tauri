---
"tauri": patch
---

Implement `raw_window_handle::RawWindowHandle` for `tauri::Window` on `Windows` and `macOS`. The `tauri::api::dialog::window_parent` function was removed since now you can use the window directly.
