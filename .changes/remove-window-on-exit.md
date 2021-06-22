---
"tauri": patch
---

Remove window object from the `Manager` internal `HashMap` on close. This fixes the behavior of using `[App|AppHandle|Window]#get_window` after the window is closed (now correctly returns `None`).
