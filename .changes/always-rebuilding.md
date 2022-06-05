---
"tauri-build": patch
---

Remove `cargo:rerun-if-changed` check for non-existent file that caused projects to _always_ rebuild.
