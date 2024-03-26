---
'tauri-bundler': 'patch:bug'
---

The NSIS uninstaller now won't mindlessly try to remove the whole installation folder when the "Remove application data" checkbox was ticked. This prevents data loss when the app was installed in a folder which contained other files.
