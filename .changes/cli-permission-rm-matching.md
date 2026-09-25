---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri permission rm <plugin>:*` and `tauri remove` no longer strip unrelated permissions whose identifier merely contains the plugin name (e.g. `tauri remove os` removing `positioner:default`). Local permission files are now looked up in the Tauri directory, like `tauri permission new`, and an unparsable permission file is skipped with a warning instead of aborting the command.
