---
'tauri-utils': 'patch:bug'
---

Serialize the CSP directive map, header source maps and plugin config with sorted keys so writing the processed config (e.g. the `tauri.conf.json` embedded in Android/iOS projects) is deterministic across builds.
