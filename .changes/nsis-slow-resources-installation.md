---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
"tauri-bundler": patch:bug
---

Fixes an issue in the NSIS installer which caused the installation to take much longer than expected when many `resources` were added to the bundle.
