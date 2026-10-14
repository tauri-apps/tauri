---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri ios xcode-script` failing to find the Tauri project when Xcode inherits package manager environment variables from `tauri ios dev --open`, or when pnpm runs the CLI directly from the `gen/apple` folder.
