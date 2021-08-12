---
"cli.rs": patch
"tauri-bundler": patch
---

Only convert package name and binary name to kebab-case, keeping the `.desktop` `Name` field with the original configured value.
