---
"tauri": minor:bug
"tauri-runtime-wry": minor:bug
---

Fixed a crash related to the `DispatcherMainThreadContext` being cloned and dropped off main thread. `WryHandle::display_handle` has a wrong lifetime now because it can't own the event loop off main thread.
