---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Fixed an issue with the CLI renaming the main executable in kebab-case when building for Windows on a non-Windows system which caused the bundler step to fail.
