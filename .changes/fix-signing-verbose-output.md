---
"tauri-bundler": patch:bug
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix verbose code signing output during `tauri build` by demoting signtool verification and signing command output to the debug log level.
