---
"tauri-runtime": patch:bug
"tauri-runtime-wry": patch:bug
---

Harden `tauri-runtime-wry` window storage with a `WindowsStore` newtype that routes all
access through fallible borrow methods, preventing recurrence of the panic class narrowly
fixed in #14862.
