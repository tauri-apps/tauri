---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
---

Added `RunEvent::ActivityRequested` on Android, emitted when the system creates an activity that is not attached to a window, for instance when the app is launched again while a foreground service kept its process alive. Pass its `activity_id` to the new `WebviewWindowBuilder::activity_id` (or `WindowBuilder::activity_id`) to attach a window to that activity.
