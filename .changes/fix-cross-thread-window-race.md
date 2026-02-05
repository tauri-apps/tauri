---
'tauri-runtime-wry': 'patch:bug'
---

Fix race condition in cross-thread window creation where Context::create_window would return before the window was inserted into the runtime's window map. This caused window_handle() calls from background threads to hang indefinitely.

The fix adds a completion channel so create_window waits for the event loop handler to signal that the window has been successfully inserted before returning.
