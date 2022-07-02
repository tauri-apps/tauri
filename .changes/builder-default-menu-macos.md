---
"tauri": patch
---

`tauri::Builder` will now include a default menu for macOS withou explicitly
using `Menu::os_default`, you can still override it through
`tauri::Builder::menu` or remove it using
`tauri::Builder::enable_macos_default_menu(false)`.
