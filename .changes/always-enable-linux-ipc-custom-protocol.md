---
"tauri": patch:breaking
"tauri-runtime-wry": patch:breaking
---

The `linux-ipc-protocol` feature is now always enabled, so the Cargo feature flag was removed.
This increases the minimum webkit2gtk version to a release that does not affect the minimum target Linux distros for Tauri apps.
