---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri build` and `tauri bundle` leaving out a `[[bin]]` whose `required-features` are enabled by the default features or by another feature.
