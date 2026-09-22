---
"tauri-runtime-wry": patch:bug
---

Fix a crash on iOS when the monitor APIs (`Window::current_monitor`, `primary_monitor`, `available_monitors`, `monitor_from_point` and the `AppHandle` equivalents) are called off the main thread, e.g. from a command: the monitor is now read on the main thread, where iOS requires it.
