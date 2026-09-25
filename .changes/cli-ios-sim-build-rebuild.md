---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri ios build` for a simulator target no longer fails with "failed to rename app: Directory not empty" when the output `.app` from a previous build exists.
