---
tauri: patch:bug
---

Fix `TrayIcon` cleanup crash when dropped off main thread by sending the inner tray icon to main thread to drop, and fixed unsoundness when cloned from multiple threads at the same time.
