---
'tauri-bundler': 'patch:bug'
---

Unset `NSISDIR` and `NSISCONFDIR` when running `makensis.exe` so it doesn't conflict with NSIS installed by the user.
