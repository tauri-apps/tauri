---
'tauri-build': 'patch:bug'
---

Emit an absolute path for the capabilities directory `cargo:rerun-if-changed`. Cargo resolves a relative watch path against the package owning the build script, while the capabilities glob is resolved against the process working directory, so callers that change the current directory before `try_build`/`try_build_context` ended up watching a non-existent directory — which is always dirty and re-ran the build script (and recompiled everything downstream) on every build.
