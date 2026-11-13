---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri migrate` failing to parse the app's `Cargo.toml` when its package version is inherited from the workspace (`version.workspace = true`).
