---
"tauri": major:breaking
"tauri-utils": major:breaking
"@tauri-apps/api": major:breaking
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Removed the deprecated `menuOnLeftClick` tray option in favor of `showMenuOnLeftClick`: `TrayIconBuilder::menu_on_left_click`, `TrayIconConfig::menu_on_left_click` (`app > trayIcon > menuOnLeftClick` in the config), `TrayIconOptions.menuOnLeftClick` and `TrayIcon.setMenuOnLeftClick` are gone. `tauri migrate` now writes `showMenuOnLeftClick` when moving a v1 `systemTray` block.
