---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri migrate` failing with `failed to parse tauri version` when there is no `Cargo.lock` and the `tauri` dependency uses a version requirement such as `"1"`, as the official v1 starter templates do.
