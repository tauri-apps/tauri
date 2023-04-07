---
'tauri': 'patch'
---

Fix `UpdaterBuilder::check` returning a parsing error when `204` is sent from server where it should instead return a `UpToDate` error.
