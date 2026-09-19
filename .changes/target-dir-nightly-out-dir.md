---
'tauri-build': 'patch:bug'
---

Resolve the target directory by walking up from `OUT_DIR` to the `build` directory instead of assuming it is exactly three levels up. Recent nightly toolchains add another level to `OUT_DIR`, which made sidecars and resources land in `target/debug/build` instead of `target/debug`.
