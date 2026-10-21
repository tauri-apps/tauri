---
"tauri-utils": major:breaking
"tauri-bundler": minor:feat
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
---

zstd compression for deb and rpm bundles accepts negative levels (down to -131072) for zstd's fast modes. `DebCompression::Zstd`'s `level` is now an `i32`, and `DebCompression::{default_level, min_level, max_level, level}` return `Option<i32>`.
