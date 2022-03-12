---
"tauri": minor
---

**Breaking change:** The `tauri://` events are no longer emitted to listeners using `Window::listen`. Use the `App::run` closure, `Window::on_window_event` and `Window::on_menu_event` instead.
