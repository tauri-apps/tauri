---
tauri-cli: patch:bug
---

Fixed an issue that caused the cli to error out with missing private key, in case the option `--no-sign` was requested and the `tauri.config` has signing key set and the plugin `tauri-plugin-updater` is used.

