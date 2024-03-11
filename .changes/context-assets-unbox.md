---
"tauri": patch:breaking
---

`Context::assets` now returns `&dyn Assets` instead of `Box<&dyn Assets>`.
