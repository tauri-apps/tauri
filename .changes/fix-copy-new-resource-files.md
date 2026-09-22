---
'tauri-build': 'patch:bug'
'tauri-utils': 'patch:enhance'
---

Emit a `cargo:rerun-if-changed` for each resource directory (and glob base directory), so that adding or removing a file inside a resource directory re-runs the build script and copies the new files. Previously only the individual files present at build time were watched, so newly added files were silently ignored until an unrelated rebuild.
