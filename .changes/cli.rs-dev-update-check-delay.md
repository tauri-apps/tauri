---
"cli.rs": patch
"cli.js": patch
---

* Remove startup delay in `tauri dev` caused by checking for a newer cli version. The check is now done upon process exit.
* Add `TAURI_SKIP_UPDATE_CHECK` env variable to skip checking for a newer CLI version.
