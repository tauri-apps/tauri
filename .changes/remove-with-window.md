---
"tauri": patch
---

Removes the `with_window` attribute on the `command` macro. Tauri now infers it using the `FromCommand` trait.
