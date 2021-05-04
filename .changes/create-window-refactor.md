---
"tauri": patch
---

The `create_window` API callback now takes two arguments: the `WindowBuilder` and the `WebviewAttributes` and must return a tuple containing both values.
