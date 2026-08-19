---
"tauri-build": "patch:bug"
---

Support cargo's build-dir layout (the default since Rust 1.100), which moves build script output from `build/<pkg>-<hash>` to `build/<pkg>/<hash>`: fixed the target directory resolution used for staging external binaries and frameworks, and the `WebView2Loader.dll` lookup for `windows-gnu` targets.
