---
"tauri-cli": "patch:bug"
---

Associate the Windows `beforeDevCommand` child process tree with a kill-on-close job object so processes are cleanly terminated when `tauri dev` is closed or terminated.
