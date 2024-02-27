---
"@tauri-apps/cli": patch:breaking
"tauri-cli": patch:breaking
"tauri": patch:breaking
"tauri-build": patch:breaking
---

The `custom-protocol` Cargo feature is no longer required on your application and is now ignored. To check if running on production, use `#[cfg(not(dev))]` instead of `#[cfg(feature = "custom-protocol")]`.
