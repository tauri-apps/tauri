---
"tauri": patch
---

Scopes the `filesystem` APIs from the webview access using `tauri.conf.json > tauri > allowlist > fs > scope`.
Scopes the `asset` protocol access using `tauri.conf.json > tauri > allowlist > protocol > assetScope`.
Scopes the `http` APIs from the webview access using `tauri.conf.json > tauri > allowlist > http > scope`.
