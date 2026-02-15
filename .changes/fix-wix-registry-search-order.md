---
"tauri-bundler": patch:bug
---

Fix WIX installer registry search order so that the named `InstallDir` key takes priority over the NSIS default key, preventing install location from changing on updates.
