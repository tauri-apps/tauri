---
'tauri': 'minor:feat'
---

Add `set_simple_fullscreen` method to `WebviewWindow`.

This method was already available on the `Window` type and is now also available on `WebviewWindow` for consistency. On macOS, it toggles fullscreen mode without creating a new macOS Space. On other platforms, it falls back to regular fullscreen.
