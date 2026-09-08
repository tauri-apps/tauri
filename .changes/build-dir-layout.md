---
"tauri-build": "patch:bug"
---

Support cargo's build-dir layout (the default since Rust 1.100), which moves build script output from `build/<pkg>-<hash>` to `build/<pkg>/<hash>`: fixed the target directory resolution used for staging external binaries and frameworks, and the `WebView2Loader.dll` lookup for `windows-gnu` targets. When `build.build-dir` is set through the `CARGO_BUILD_BUILD_DIR` environment variable, staged artifacts now follow the executable into the target directory instead of the build directory.
