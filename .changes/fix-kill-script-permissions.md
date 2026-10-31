---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri-stop-dev-processes.sh` created without execute permission on macOS, causing `beforeDevCommand` child processes to survive after closing the app.
