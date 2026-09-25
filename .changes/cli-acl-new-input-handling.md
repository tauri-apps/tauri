---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri capability new` and `tauri permission new` now accept an `--out` path to a file that does not exist yet, trim comma-separated prompt answers (so `fs:default, core:default` works), report invalid permissions as errors instead of panicking, and reject identifiers that are not valid file names (such as `../../x`), which previously let them write outside of the capabilities or permissions directory.
