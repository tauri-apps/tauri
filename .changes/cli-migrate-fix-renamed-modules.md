---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri migrate` incorrectly migrating `@tauri-apps/api/tauri` module to just `core` and `@tauri-apps/api/window` to just `webviewWindow`.
