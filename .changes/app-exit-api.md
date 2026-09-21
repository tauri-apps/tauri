---
'tauri': 'minor:feat'
'@tauri-apps/api': 'minor:feat'
---

Added the `exit` function to `@tauri-apps/api/app`, backed by the new `plugin:app|exit` command (`core:app:allow-exit` permission), to exit the app without requiring the `@tauri-apps/plugin-process` plugin.
