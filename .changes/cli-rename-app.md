---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Fixed an issue that caused the CLI to rename app binaries incorrectly if the product name contained a `.` which resulted in the bundling step to fail.
