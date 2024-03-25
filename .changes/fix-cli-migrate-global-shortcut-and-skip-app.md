---
"tauri": patch:bug
---

Fix multiple runs of migration and some package name changes.
- Skip all core plugin migrations.
- Don't exit when `tauri::Builder` is not found.
- Skip overwriting existing migrated capabilities.
- `globalShortcut` now migrates to `global-shortcut`.
