---
"tauri": "patch:bug"
"tauri-bundler": "patch:bug"
---

Fix NSIS installers not blocking downgrades in silent (`/S`), passive (`/P`) and update (`/UPDATE`) modes when `bundle.windows.allowDowngrades` is `false`. The installed version is now compared in `.onInit`, so the check no longer depends on the reinstall page, which never runs in silent mode. Silent installers print the `silentDowngrades` message to the console, passive and update installers show it in a message box, and all of them exit with code `2`.
