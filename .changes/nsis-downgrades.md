---
'tauri-bundler': 'patch'
---

Fix NSIS installer disabling `do not uninstall` button and silent installer aborting, if `allowDowngrades` was disabled even when we are not downgrading.
