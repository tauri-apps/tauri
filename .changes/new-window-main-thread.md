---
"tauri": minor:changes
"tauri-runtime-wry": minor:changes
---

The new window handler passed to `on_new_window` no longer requires `Sync`, and runs on main thread on Windows, aligning with other platforms
