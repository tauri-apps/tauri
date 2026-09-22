---
"tauri-runtime-cef": patch:bug
---

Fixed `Window::set_fullscreen` and `set_fullscreen_on_monitor` going wrong on macOS when called while the window was still animating its previous fullscreen transition: AppKit drops the toggle then, leaving the runtime's idea of the fullscreen state out of step with the window (a request to leave fullscreen was lost, and the next request to enter left it instead). Requests made during a transition are now applied once it ended.
