---
"tauri": patch:breaking
---

`Window::close` now triggers `RunEvent::CloseRequested` instead of forcing the window to be closed.
