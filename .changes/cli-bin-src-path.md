---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix binaries in `src/bin` and `src/main.rs` being left out of bundles when `Cargo.toml` declares a `[[bin]]` target without a `path`.
