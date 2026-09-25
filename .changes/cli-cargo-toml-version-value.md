---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix the Cargo.toml feature rewrite corrupting a string dependency version that has a trailing comment or uses single quotes (e.g. `tauri = "2" # pin` became `"2#pin"`). The version value is now kept as-is and the comment is preserved.
