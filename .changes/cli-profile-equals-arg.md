---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `--release` being added to the cargo command when a custom profile is passed as `--profile=<name>`, which made cargo reject the build.
