---
tauri-build: patch:bug
---

Switch back to `static_vcruntime` crate, this fixes a linking error when the crate using `tauri_build::build` is used as library.
