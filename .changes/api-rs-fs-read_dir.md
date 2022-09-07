---
"api": "major"
"tauri": "major"
---

**Breaking change** `tauri::api::dir::read_dir()` signature has changed. It no longer has `recursive` option and now returns `DirEntry` struct instead of the removed `DiskEntry` struct.
