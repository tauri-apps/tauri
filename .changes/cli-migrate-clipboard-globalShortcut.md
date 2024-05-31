---
'tauri-cli': 'patch:bug'
---

Correct module name resolution for `clipboard` and `globalShortcut` plugins.

| V1 module name | wrong migration | correct,new migration |
| -------------- | --------------- | --------------------- |
| @tauri-apps/api/clipboard | @tauri-apps/plugin-clipboard | @tauri-apps/plugin-clipboard-manager
| @tauri-apps/api/globalShortcut | @tauri-apps/globalShortcut | @tauri-apps/plugin-global-shortcut
