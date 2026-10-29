---
"tauri": "minor:feat"
---

Add the menu API to iOS, showing the app menu in the iPadOS 26 menu bar. Earlier versions use it for key commands and the keyboard shortcuts list. `Builder::menu`, `AppHandle::set_menu`, `AppHandle::remove_menu`, `on_menu_event` and `RunEvent::MenuEvent` are now available on iOS, native menu icons map to SF Symbols, and `Menu::default` creates the same menus as on macOS. Context menus (`popup`, `popup_at`, `Window::popup_menu` and `Window::popup_menu_at`) are shown as edit menus on iOS 16 and later. Window menus remain desktop-only.
