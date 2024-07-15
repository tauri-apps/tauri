---
"tauri-bundler": "patch:changes"
---

Make `NSIS_HOOK_PREINSTALL` and `NSIS_HOOK_PREUNINSTALL` run before `CheckIfAppIsRunning` (which checks if the app is running and asks the user if they want to kill the app)
