---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
---

Fixes Android dev and build commands reading `tauri.ios.conf.json` instead of `tauri.android.conf.json` to merge platform-specific configuration.
