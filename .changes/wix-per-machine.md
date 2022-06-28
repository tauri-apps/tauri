---
"tauri-bundler": "patch"
"cli.rs": "patch"
"cli.js": "patch"
---

Added `perMachine` option to `tauri.conf.json > tauri > bundle > windows > wix`, disabled by default.
**Breaking change** `msi` installer will install your app per-user by default now.

