---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix signing with empty string password failing with "Wrong password for that key". Empty passwords now correctly generate unencrypted keys.
