---
"tauri-runtime-cef": patch:bug
---

Custom protocol handlers, the `ipc` handler included, now run on the main thread like they do with the wry runtime, instead of on a thread per request. Commands that wait on the main thread — creating a menu or a menu item, for one — deadlocked with the event loop delivering run events, and commands that dropped main-thread-only objects, such as removing a tray icon on macOS, crashed the app.
