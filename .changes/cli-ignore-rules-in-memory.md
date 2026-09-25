---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

The CLI no longer writes its default ignore rules to shared files in the system temporary directory (`.gitignore` and `.tauri/.gitignore`) and reads them back, which let other local users change which files the project lookup and the dev watcher skip. The rules are now applied in memory. `TAURI_CLI_WATCHER_IGNORE_FILENAME` is now consistently treated as an ignore file name looked up in each directory, as documented.
