---
"tauri": "patch:enhance"
"@tauri-apps/api": "patch:enhance"
---

`IconMenuItem::set_native_icon` and `Submenu::set_native_icon` are no longer macOS-only. Known `NativeIcon` variants now map to freedesktop icon names on Linux and stock shell icons on Windows, the same as `TrayIcon::set_native_icon`.
