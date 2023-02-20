---
'cli.rs': 'minor'
'tauri-build': 'minor'
---

Added support for Cargo's workspace inheritance for package information. The cli now also detects inherited `tauri` and `tauri-build` dependencies and disables manifest rewrites accordingly.
