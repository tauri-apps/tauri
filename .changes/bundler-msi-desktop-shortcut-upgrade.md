---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix the MSI installer re-creating the desktop shortcut on every update even after the user deleted it. The shortcut is now only re-created during an upgrade if it still exists, matching the NSIS installer.
