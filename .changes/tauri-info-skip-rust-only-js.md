---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri info` reporting the Rust-only plugins (`localhost`, `persisted-scope` and `single-instance`) as missing their `@tauri-apps/plugin-*` JavaScript package, which is never published for those plugins.
