---
"tauri-bundler": "patch:bug"
---

Make the `Glob` and `GlobPattern` error variants available on all platforms, fixing a compile error in the Windows bundler utilities on targets other than Windows, macOS and Linux.
