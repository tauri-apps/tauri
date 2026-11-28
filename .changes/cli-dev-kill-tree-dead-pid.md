---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri dev` leaving a `sh` process spinning at full CPU on macOS after the app was quit on its own (for example with Cmd+Q). The process tree walk now stops when the app has already exited and only follows real child processes.
