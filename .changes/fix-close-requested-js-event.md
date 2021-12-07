---
"tauri": patch
---

Prevent window closing if `tauri://close-requested` is listened on the JS layer. Users must call `appWindow.close()` manually when listening to that event.
