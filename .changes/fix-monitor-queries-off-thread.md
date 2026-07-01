---
'tauri-runtime-wry': 'patch:bug'
---

Query monitor information (`primary_monitor`, `monitor_from_point`, `available_monitors`) on the main thread from the app-level runtime handle instead of touching the event loop's window target directly. The window target is not thread safe, so calling these from a background thread could crash the process (segfault / heap corruption).
