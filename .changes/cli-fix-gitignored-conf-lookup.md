---
"cli.rs": patch
"cli.js": patch
---

Check if `$CWD/src-tauri/tauri.conf.json` exists before walking through the file tree to find the tauri dir in case the whole project is gitignored.
