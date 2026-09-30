---
"tauri": "minor:feat"
"@tauri-apps/api": "minor:feat"
---

Add `TrayIconBuilder::native_icon` and `TrayIcon::set_native_icon` (the `nativeIcon` option of the JavaScript `TrayIcon.new` and `TrayIcon.setNativeIcon`), to use a platform-native icon such as `NativeIcon::StatusAvailable` for the tray icon. Known variants map to freedesktop icon names on Linux, AppKit image names on macOS and stock shell icons on Windows, so the icon follows the system theme instead of being shipped with the app.
