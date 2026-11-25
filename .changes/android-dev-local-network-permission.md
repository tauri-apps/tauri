---
'tauri': 'patch:bug'
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Fix `tauri android dev` failing to load the frontend on physical Android 17 devices. The app could not reach the dev server on the network address because Android 17 requires the `ACCESS_LOCAL_NETWORK` permission. Debug builds now declare this permission and the CLI grants it.
