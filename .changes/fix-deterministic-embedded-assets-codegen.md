---
'tauri-codegen': 'patch:bug'
---

Emit embedded assets and CSP script/style hashes in sorted order so `generate_context!` output no longer depends on the filesystem walk order, which varies across machines and broke reproducible builds.
