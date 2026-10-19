---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Skip the `--target` validation when `rustup target list` fails, e.g. when `rustup` is on `PATH` but has no toolchain installed, instead of rejecting every target as missing.
