---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
---

Fixes Cargo features and args not being applied to the first cargo build calls of `[android|ios] [dev|build]` commands.
