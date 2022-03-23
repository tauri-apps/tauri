---
"tauri": patch
---

**Breaking change:** The `Window::parent_window` method now returns *HWND* instead of *c_void* on Windows.
