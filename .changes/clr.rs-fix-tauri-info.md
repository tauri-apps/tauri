---
"cli.rs": patch
---

Fix `tauri info`
* Properly detect `yarn` and `npm` versions on windows.
* Fix a panic caused by a wrong field name in `metadata.json`
