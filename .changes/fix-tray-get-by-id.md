---
"tauri": patch:bug
---

Fix `TrayIcon.getById` returning a new resource ID instead of reusing a previously created id from `TrayIcon.new`.
