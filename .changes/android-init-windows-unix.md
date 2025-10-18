---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': patch:bug
---

Strip Windows-only extensions from the binary path so an Android project initialized on Windows can be used on UNIX systems.
