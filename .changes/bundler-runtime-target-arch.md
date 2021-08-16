---
"tauri-bundler": patch
---

Check target architecture at runtime running `$ RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json` and parsing the `llvm-target` field, fixing macOS M1 sidecar check until we can compile the CLI with M1 target on GitHub Actions.
