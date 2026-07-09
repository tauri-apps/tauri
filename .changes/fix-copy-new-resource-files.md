---
'tauri-build': 'patch:bug'
'tauri-utils': 'patch:enhance'
---

Emit a `cargo:rerun-if-changed` for each resource directory and glob base directory, so adding or removing files inside configured resource directories re-runs the build script and copies the changed resources.
