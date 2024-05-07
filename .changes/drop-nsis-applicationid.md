---
'tauri-bundler': 'patch:enhance'
---

Use nsis's built-in COM plugin instead of `ApplicationID` plugin, this reduces the installer size by 100 KB, and also fixes pinned shortcut not getting cleaned up on uninstall.
