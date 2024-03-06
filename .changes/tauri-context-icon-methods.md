---
'tauri': 'major:breaking'
---

Removed `Context::default_window_icon_mut` and `Context::tray_icon_mut`, use `Context::set_default_window_icon` and `Context::set_tray_icon` instead. Also changed `Context::set_tray_icon` to accept `Option<T>`.
