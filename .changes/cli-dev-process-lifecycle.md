---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri dev` now reports an error instead of panicking when the `beforeDevCommand` cannot be spawned or the `devUrl` host cannot be resolved, and no longer risks stopping the `beforeDevCommand` process tree twice when Ctrl+C and the app exit race.
