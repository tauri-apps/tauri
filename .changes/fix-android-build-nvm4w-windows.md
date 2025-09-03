---
'tauri-cli': 'patch:bug'
---

Fix Android build error on Windows when using nvm4w (issue documented at #13892 [bug] [android] [windows] [cli] [build] Android Build Error: A problem occurred starting process 'command 'C:\nvm4w\nodejs\node.exe.cmd'' and Error: Cannot find module '...tauri' when using nvm4w). The BuildTask.kt template now includes robust fallback logic for Windows executable detection, preventing the "Cannot find module" and "node.exe.cmd" errors that occurred with nvm4w Node.js installations. The fix tries multiple Windows-specific fallbacks including .cmd, .bat extensions and cargo as a last resort.
