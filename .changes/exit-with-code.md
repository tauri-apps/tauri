---
'tauri-runtime-wry': 'patch:bug'
---

Transfer the exit code from the `window.app_handle().exit(1)` call to the `run_return()` result instead of always returning 0.
