---
"tauri-runtime-wry": "patch:bug"
---

Fixed the app crashing with a non-unwinding panic during window creation when `width`, `height`, `minWidth`, `minHeight`, `maxWidth` or `maxHeight` was set to a value greater than `i32::MAX` (e.g. on macOS this surfaced as an `NSWindow` frame assertion). These dimensions are now clamped to `i32::MAX` instead of being passed through unchecked.
