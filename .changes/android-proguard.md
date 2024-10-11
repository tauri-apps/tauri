---
"tauri": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix android invalid proguard file when using an `identifier` that contains a component that is a reserved kotlin keyword, like `in`, `class`, etc
