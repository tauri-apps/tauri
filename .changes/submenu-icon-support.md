---
'@tauri-apps/api': 'minor:enhance'
'tauri': 'minor:enhance'
---

Added icon (icon and nativeIcon) support for Submenu:
- In the Rust API (`tauri`), you can now set an icon for submenus via the builder and dedicated methods.
- In the JS/TS API (`@tauri-apps/api`), `SubmenuOptions` now has an `icon` field, and the `Submenu` class provides `setIcon` and `setNativeIcon` methods.
- Usage examples are added to the documentation and demo app.

This is a backwards-compatible feature. Submenus can now display icons just like regular menu items.
