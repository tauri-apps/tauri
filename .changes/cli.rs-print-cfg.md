---
"cli.rs": patch
---

Replaces usage of the private command `RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json` with the stable and public command `rustc --print cfg`.
