---
"tauri": patch:bug
---

`AppHandle::restart()` now waits for `RunEvent::Exit` to be delivered before restarting the application.
