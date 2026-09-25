---
"tauri": patch:bug
---

Stop queuing large channel payloads once the target webview is closed. They were kept in memory until the app exited, and a new webview reusing the same label could fetch them.
