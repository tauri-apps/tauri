---
"tauri": patch
---

**Breaking change:** The `Window::hwnd` method now returns *HWND* from `windows-rs` crate instead of *c_void* on Windows.
