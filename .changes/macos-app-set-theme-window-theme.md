---
"tauri-runtime-wry": patch:bug
---

On macOS, `App::set_theme` now also updates the theme each window reports, so `Window::theme` (and `getCurrentWindow().theme()` in JavaScript) reflects the app-level theme instead of keeping the previous value until the system appearance changes.
