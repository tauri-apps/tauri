---
'tauri': 'patch:bug'
---

Ensure initalize logic and dropping of menu item is done on the main thread, this fixes the crash when a menu item is dropped on another thread.
