---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri init` using a raw string instead of a `format!` escape for the `tauri-utils` and `tauri-plugin` dependency fallbacks, which would write `{{ version = "2" }}` instead of `{ version = "2" }` into the `[patch.crates-io]` section of the generated `Cargo.toml`.
