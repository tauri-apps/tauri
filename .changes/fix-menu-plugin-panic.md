---
"tauri": patch:bug
"tauri-macros": patch:enhance
---

Fix a panic (process abort) in the menu plugin when a command receives an unexpected `ItemKind` (for example a root `Menu` passed through `Menu.new({ items: [...] })` or a raw IPC call). The menu commands now dispatch through a new `do_menu_item_checked!` macro that returns a recoverable error instead of hitting `unreachable!()`. The existing `do_menu_item!` macro is left unchanged.
