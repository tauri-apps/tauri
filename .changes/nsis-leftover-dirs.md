---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
"tauri-bundler": patch:bug
---

Fixes an issue in the NSIS installer which caused the uninstallation to leave empty folders on the system if the `resources` feature was used.
