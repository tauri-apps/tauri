---
"tauri-build": 'patch:bug'
---

Fixed an issue that caused the resource compiler to not run on Windows when `package.version` was not set in `tauri.conf.json` preventing the app from starting.
