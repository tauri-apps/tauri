---
"tauri-cli": patch:enhance
---

Properly read the `required-features` field of binaries in Cargo.toml to prevent bundling issues when the features weren't enabled.
