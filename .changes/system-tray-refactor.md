---
"tauri": patch
---

**Breaking change**: The `system_tray` and `on_system_tray_event` APIs were not designed to have all the new features: submenus, item updates, click events, positioning... so we broke it before going to stable.
