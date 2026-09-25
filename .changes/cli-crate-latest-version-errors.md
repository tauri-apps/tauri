---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri info` now logs a warning when it cannot check the latest crate version on crates.io instead of silently ignoring the failure, and no longer panics if crates.io returns a version it cannot parse.
