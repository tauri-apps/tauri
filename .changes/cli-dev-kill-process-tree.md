---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri dev` now kills the whole process tree of the running app when it restarts or exits, so processes spawned by the app (e.g. sidecars) are no longer left orphaned.
