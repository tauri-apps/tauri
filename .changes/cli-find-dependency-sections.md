---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix Cargo.toml feature injection and the v1 migration only updating either `[dependencies]` or `[target.'cfg(..)'.dependencies]`, whichever came first in the file. Both the main and all target-specific dependency tables are now updated.
