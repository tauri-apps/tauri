---
"tauri-utils": patch:breaking
"tauri-cli": patch:breaking
"@tauri-apps/cli": patch:breaking
---

Changed the capability format to allow configuring both `remote: { urls: Vec<String> }` and `local: bool (default: true)` instead of choosing one on the `context` field.
