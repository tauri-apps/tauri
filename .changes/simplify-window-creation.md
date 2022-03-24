---
"tauri-runtime-wry": minor
---

Use a random window id instead of `tao::window::WindowId` to not block the thread waiting for the event loop to process the window creation.
