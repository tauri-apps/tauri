---
"tauri-runtime": patch:bug
"tauri-runtime-wry": patch:bug
---

Refactor RefCell usage for `tauri-runtime-wry` internal `WindowsStore` to prevent panics
on previously unchecked borrows.
