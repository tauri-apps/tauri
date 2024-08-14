---
"tauri-runtime-wry": patch:bug
---

Ensure system tray is created when the event loop is ready. Menu item modifications are not applied unless it is ready.
If you need to modify the menu items immediately after creating a tray in the setup hook,
either directly configure the menu item change when creating the menu or move the code when `RunEvent::Ready` is fired.
See https://docs.rs/tauri/latest/tauri/struct.App.html#method.run for more information.
