---
"tauri-bundler": patch
---

Change WiX MajorUpgrade element's `Schedule` to `afterInstallExecute` to prevent removal of existing configuration such as the executable's pin to taskbar.
