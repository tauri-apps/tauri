---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

CLI commands will now consistently search for the `app_dir` (the directory containing `package.json`) from the current working directory of the command invocation.
