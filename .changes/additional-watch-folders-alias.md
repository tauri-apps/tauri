---
"tauri-utils": patch:bug
---

Added `additional-watch-folders` as an alias for the `build > additionalWatchFolders` configuration value, so the kebab-case spelling that matches the camelCase key is accepted in `Tauri.toml`. The previous `additional-watch-directories` alias keeps working.
