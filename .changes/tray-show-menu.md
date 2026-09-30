---
"tauri": "minor:feat"
"@tauri-apps/api": "minor:feat"
---

Add `TrayIcon::show_menu` (`TrayIcon.showMenu` in JavaScript), which shows the tray menu at the current cursor position, so an app that disabled `showMenuOnLeftClick` and `showMenuOnRightClick` can decide when the menu shows up, for instance after updating its items.
