---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `bundle > android > debugApplicationIdSuffix` being written to the `signingConfigs` debug block instead of the `buildTypes` one, and keep the existing content of single-line debug blocks such as `getByName("debug") { isDebuggable = true }` instead of dropping it.
