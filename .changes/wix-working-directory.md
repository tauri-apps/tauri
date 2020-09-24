---
"tauri-bundler": patch
---

Set the Windows installer (WiX) `WorkingDirectory` field to `INSTALLDIR` so the app can read paths relatively (previously resolving to `C:\Windows\System32`).
