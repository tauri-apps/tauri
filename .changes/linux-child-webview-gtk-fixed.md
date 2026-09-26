---
"tauri-runtime-wry": patch:bug
---

Child webviews on Linux are mounted in a `GtkFixed`, so `set_bounds` (position and size) is honored and multiwebview layouts work, including on Wayland.
