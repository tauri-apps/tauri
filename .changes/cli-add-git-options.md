---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri add` now honors `--tag`, `--rev` and `--branch` for official plugins instead of silently installing the registry version, and rejects passing more than one of them. With npm, the JS package requirement is now `~<version>` like the other package managers, instead of `>=<version>` which allowed a later major version.
