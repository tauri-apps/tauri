---
'tauri': 'major:breaking'
---

The tray icon and menu have received a huge refactor with a lot of breaking changes in order to add new functionalities and improve the DX around using them and here is an overview of the changes:

- All menu and tray types are now exported from `tauri::menu` and `tauri::tray` modules with new names so make sure to check the new types.
- Removed `tauri::Builder::system_tray`, instead you should use `tauri::tray::TrayIconBuilder` inside `tauri::Builder::setup` hook to create your tray icons.
- Changed `tauri::Builder::menu` to be a function to accomodate for new menu changes, you can passe `tauri::menu::Menu::default` to it to create a default menu.
- Renamed `tauri::Context` methods `system_tray_icon`, `tauri::Context::system_tray_icon_mut` and `tauri::Context::set_system_tray_icon` to `tauri::Context::tray_icon`, `tauri::Context::tray_icon_mut` and `tauri::Context::set_tray_icon` to be consistent with new type names.
- Added `RunEvent::MenuEvent` and `RunEvent::TrayIconEvent`.
- Added `App/AppHandle::set_menu`, `App/AppHandle::remove_menu`, `App/AppHandle::show_menu`, `App/AppHandle::hide_menu` and `App/AppHandle::menu` to access, remove, hide or show the app-wide menu that is used as the global menu on macOS and on all windows that don't have a specific menu set for it on Windows and Linux.
- Added `Window::set_menu`, `Window::remove_menu`, `Window::show_menu`, `Window::hide_menu`, `Window::is_menu_visible` and `Window::menu` to access, remove, hide or show the menu on this window.
- Added `Window::popup_menu` and `Window::popup_menu_at` to show a context menu on the window at the cursor position or at a specific position. You can also popup a context menu using `popup` and `popup_at` methods from `ContextMenu` trait which is implemented for `Menu` and `Submenu` types.
- Added `App/AppHandle::tray`, `App/AppHandle::tray_by_id`, `App/AppHandle::remove_tray` and `App/AppHandle::remove_tray_by_id` to access or remove a registered tray.
- Added `WindowBuilder/App/AppHandle::on_menu_event` to register a new menu event handler.
- Added `App/AppHandle::on_tray_icon_event` to register a new tray event handler.
