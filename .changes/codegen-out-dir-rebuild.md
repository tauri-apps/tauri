---
"tauri-codegen": patch:bug
---

Set the modification time of the asset, icon and macOS `Info.plist` files `generate_context!` writes to `OUT_DIR` to 2000-01-01 UTC, so the first `cargo build` without changes after a build that created them no longer recompiles the app crate.
