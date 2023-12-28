---
'tauri': 'patch:breaking'
---

`tauri::scope` module is recieving a couple of consistency changes:

- Added `tauri::scope::fs` module.
- Removed `scope::IpcScope` re-export, use `scope::ipc::Scope`.
- Removed `FsScope`, `GlobPattern` and `FsScopeEvent`, use `scope::fs::Scope`, `scope::fs::Pattern` and `scope::fs::Event` respectively.
