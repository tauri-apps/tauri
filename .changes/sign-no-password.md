---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix updater signing private keys generated using `tauri signer generate` with empty password can't be used (The keys generated during tauri were broken between v2.9.3 and v2.10.0, you'll need to regenerate them)
