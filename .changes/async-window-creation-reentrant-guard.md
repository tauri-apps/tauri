---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
---

Added `WindowBuilder::build_async`, `WebviewWindowBuilder::build_async`, and `Window::add_child_async` for non-blocking creation, along with `Error::ReentrantWindowCreation` to reject re-entrant synchronous window/webview creation inside main-thread callbacks instead of deadlocking.
