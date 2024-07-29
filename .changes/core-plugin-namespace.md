---
"tauri": patch:breaking
"tauri-plugin": patch:breaking
"@tauri-apps/cli": patch:breaking
"tauri-cli": patch:breaking
---

Core plugin permissions are now prefixed with `core:` and the `core` plugin name is reserved.
The `tauri migrate` tool will automate the migration process, which involves prefixing all `app`, `event`, `image`, `menu`, `path`, `resources`, `tray`, `webview` and `window` permissions with `core:`.
