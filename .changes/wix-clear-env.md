---
"tauri-bundler": patch
---

Clear environment variables on the WiX light.exe and candle.exe commands to avoid "Windows Installer Service could not be accessed" error. Variables prefixed with `TAURI` are propagated.
