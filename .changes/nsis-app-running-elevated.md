---
"tauri-bundler": patch:enhance
---

Show a dedicated message in the NSIS installer when the running app can't be closed because it runs with higher privileges than the installer (for example as administrator), instead of offering to kill it and then failing with "Failed to kill".
