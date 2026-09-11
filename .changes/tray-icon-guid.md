---
"tauri": minor:feat
---

Add `TrayIconBuilder::guid` to register the tray icon with a stable `NOTIFYICONDATA.guidItem` on Windows, so the user's "always show in the taskbar" setting survives updates that move the executable. Bumps `tray-icon` to 0.25.
