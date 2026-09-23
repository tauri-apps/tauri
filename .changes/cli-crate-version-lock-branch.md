---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri info` now lists every locked version of a Rust crate when `Cargo.lock` contains more than one, and no longer panics when the crates.io response cannot be parsed.
