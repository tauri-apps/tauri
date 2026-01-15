---
tauri: minor:feat
tauri-runtime: minor:feat
tauri-runtime-wry: minor:feat
---

**Linux**: Add GTK4 and WebKitGTK 6.0 support.

Breaking changes for Linux:
- `temp_dir_path` methods removed from `TrayIconBuilder` and `TrayIcon` (ksni uses DBus)
- Requires GTK4 4.6+ and WebKitGTK 6.0+
