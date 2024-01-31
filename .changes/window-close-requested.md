---
"tauri": patch:breaking
"@tauri-apps/api": patch:breaking
---

`Window::close` now triggers a close requested event instead of forcing the window to be closed.
