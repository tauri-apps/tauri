---
"tauri": patch
---

Callbacks passed to `tauri::plugin::Builder::setup` or `tauri::plugin::Builder::setup_with_config` are no longer required to implement `Sync`.
