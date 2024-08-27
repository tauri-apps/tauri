---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Fix the `add` command NPM version specifier for known plugins from `2.0.0-rc` (unknown version requirement) to `^2.0.0-rc`.
