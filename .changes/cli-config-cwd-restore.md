---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix the CLI's working directory being left changed to the config directory when the Tauri configuration fails to parse, for example when `tauri dev` reloads an invalid config.
