---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri dev` not reacting to the app exiting when a process spawned by the app kept its stderr open, and stop keeping the whole app stderr output in memory. Also fix command output capture that could return empty output when the command finished before its output was read.
