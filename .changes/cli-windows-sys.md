---
"tauri-cli": "patch"
---

Use `windows-sys` crate instead of `winapi` which fixes installing the published cli from crates.io using `cargo install tauri-cli --version "^2.0.0-beta"`.
