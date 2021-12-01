---
"tauri": patch
---

Scopes the `filesystem` APIs from the webview access using `tauri.conf.json > tauri > allowlist > fs > scope`.
Scopes the `asset` protocol access using `tauri.conf.json > tauri > allowlist > protocol > assetScope`.
Scopes the `http` APIs from the webview access using `tauri.conf.json > tauri > allowlist > http > scope`.
Scopes the `shell` execute API from the webview access using `tauri.conf.json > tauri > allowlist > shell > scope`. Additionally, check the `tauri.conf.json > tauri > bundle > externalBin` to prevent access to unknown sidecars.
