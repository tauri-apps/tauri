---
"tauri-build": patch:bug
---

The `.ico` from `bundle > icon` (or the `icons/icon.ico` fallback) embedded as the Windows application icon is now resolved relative to the config file directory instead of the build script's working directory, so it works with `Attributes::config_path`.
