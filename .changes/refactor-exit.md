---
"tauri": patch:breaking
---

`AppHandle::exit` and `AppHandle::restart` now go triggers `RunEvent::ExitRequested` and `RunEvent::Exit` and cannot be executed on the event loop handler.
