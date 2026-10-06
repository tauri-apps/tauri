---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Fixed `tauri migrate` failing with `failed to parse tauri version` when the `tauri` dependency in `Cargo.toml` is a version requirement such as `"1"` and there is no `Cargo.lock`.
