---
'tauri-bundler': 'patch:bug'
---

Fix bundler skipping updater artifacts if `updater` target shows before other updater-enabled targets in the list, see [#7349](https://github.com/tauri-apps/tauri/issues/7349).
