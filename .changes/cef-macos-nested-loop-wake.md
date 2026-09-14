---
"tauri-runtime-cef": patch:bug
---

Fixed a panic on macOS ("tried to handle event while another event is currently being handled") when a `run_on_main_thread` closure or a window event handler spins a nested AppKit event loop, such as a context menu shown with `tauri::menu::ContextMenu::popup` or a modal dialog. Event-loop wake-ups are now delivered from a run-loop observer at the start of a main-loop pass and only once no `ApplicationHandler` callback is running, so winit's proxy is never performed inside such a nested loop.
