---
"tauri-bundler": patch
---

Replaces usage of the nightly command `RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json` with the stable command `rustc --print cfg`, improving target triple detection.
