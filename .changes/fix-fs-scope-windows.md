---
"tauri": patch
---

Allow the canonical, absolute form of a path for the filesystem scope on Windows if `std::fs::canonicalize` returns a path, fallback to `\\?\$PATH`.
