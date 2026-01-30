---
"tauri-bundler": patch:bug
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Updated `nsis_tauri_utils` to 0.5.3:

- Use an alternative method `CreateProcessWithTokenW` to run programs as user, this fixed a problem that the program launched with the previous method can't query its own handle
