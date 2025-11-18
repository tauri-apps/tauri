---
tauri-cli: patch:bug
---

Fixed an issue that caused the cli to print errors like `Error Failed to parse version 2 for crate tauri` when there was no `Cargo.lock` file present yet. This will still be logged in `--verbose` mode.
