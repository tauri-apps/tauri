---
"tauri": patch
---

**Breaking:** `api::path::resolve_path()` and `api::path::app_dir()` now takes the config as first argument.
**Breaking:** `api::path::app_dir()` now resolves to `${configDir}/${config.tauri.bundle.identifier}`.
