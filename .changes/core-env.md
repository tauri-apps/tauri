---
"tauri": patch
---

The `process`, `path` and `updater` APIs now takes a `tauri::Env` argument, used to force environment variables load on startup to prevent env var update attacks.
