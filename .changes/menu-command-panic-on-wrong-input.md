---
tauri: minor:bug
tauri-macros: minor:bug
---

Fix menu related commands can panic if called with invalid input through `invoke` directly.

- The internal `do_menu_item!` macro now returns `Err(crate::Error::UnexpectedMenuKind)` instead of `unreachable!()`
- Added a new error type `tauri::Error::UnexpectedMenuKind`
- `menu:new` with the `Predefined` kind now returns an error instead of panicking when the `item` option is missing
- Converting an `Image` to a menu or tray icon now returns `tauri::Error::InvalidIcon` when the RGBA buffer does not match the image size, instead of panicking on Linux when the icon is rendered
