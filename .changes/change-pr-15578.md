---
"tauri": patch:enhance
---

`State` had `Send` and `Sync` trait bounds that were already implied, remove them from the struct definition.
