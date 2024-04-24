---
'tauri-bundler': 'patch:enhance'
---

Use nsis's built-in com plugin instead of ApplicationID plugin, this reduces the installer size by 150-200 KB, and also fixes pinned shortcut not getting cleaned up on uninstall
