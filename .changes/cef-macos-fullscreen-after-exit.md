---
"tauri-runtime-cef": patch:bug
---

Fixed `Window::set_fullscreen(true)` and `set_fullscreen_on_monitor` being silently dropped on macOS when called right after the window left fullscreen: AppKit refuses the toggle while the previous transition is still winding down, and winit reverts to a windowed state instead of retrying. The runtime now re-applies the request until the window is fullscreen.
