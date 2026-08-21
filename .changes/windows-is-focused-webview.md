---
"tauri": patch:bug
"tauri-runtime-wry": patch:bug
---

Fix `Window::is_focused` always returning `false` on Windows when the window hosts a webview, which also made `Manager::get_focused_window` return `None`.
