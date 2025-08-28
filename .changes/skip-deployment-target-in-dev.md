---
tauri-build: patch:enhance
tauri-cli: patch:enhance
'@tauri-apps/cli': patch:enhance
---

Tauri now ignores `macOS.minimumSystemVersion` in `tauri dev` to prevent forced rebuilds of macOS specific dependencies when using something like `rust-analyzer` at the same time as `tauri dev`.
