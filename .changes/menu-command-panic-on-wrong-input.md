---
tauri: minor:bug
tauri-macros: minor:bug
---

Fix menu related commands can panic if called with invalid menu types through `invoke` directly. The internal `do_menu_item!` macro now returns `Err(crate::Error::UnexpectedMenuKind)` instead of `unreachable!()`
