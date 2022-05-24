---
"tauri-runtime-wry": patch
---

Fixes a crash when using the menu with the inspector window focused on macOS. In this case the `window_id` will be the id of the first app window.
