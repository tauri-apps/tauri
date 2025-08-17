---
"tauri": 'minor:enhance'
---

Introduce `with_inner_tray_icon` for Tauri `TrayIcon` to access the inner platform-specific tray icon.

Note that `tray-icon` crate may be updated in minor releases of Tauri.
Therefore, it’s recommended to pin Tauri to at least a minor version when you’re using `with_inner_tray_icon`.
