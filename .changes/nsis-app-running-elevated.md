---
"tauri-bundler": patch:enhance
---

Show a dedicated message in the NSIS installer when the running app can't be closed because the installer doesn't have permission to close it (for example when the app runs as administrator), instead of offering to kill it and then failing with "Failed to kill".
