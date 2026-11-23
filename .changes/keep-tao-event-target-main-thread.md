---
"tauri": minor:bug
"tauri-runtime-wry": minor:bug
---

Fixed a crash related to the `DispatcherMainThreadContext` being cloned and dropped off main thread. `WryHandle::display_handle` has a wrong lifetime now because it can't own the event loop off main thread. It now goes through the event loop like `window_handle`, so calling it off the main thread blocks until the main thread handles the request.
