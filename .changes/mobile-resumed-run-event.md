---
"tauri-runtime-wry": "patch:bug"
---

Emit `RunEvent::Resumed` on mobile when the application is resumed. Previously the resume was only dispatched as a per-window event, so it was silently dropped when no windows existed (e.g. on Android after the activity was destroyed while a foreground service kept the process alive).
