---
'tauri-cli': 'patch:enhance'
'@tauri-apps/cli': 'patch:enhance'
---

Add `--no-bundle` flag for `tauri build` command to skip bundling. Previously `none` was used to skip bundling, it will now be treated as invalid format and a warning will be emitted instead.
