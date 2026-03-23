---
"tauri-bundler": patch:bug
---

Fix AppImage `.DirIcon` and `.desktop` symlinks using absolute paths instead of relative paths, which broke icon display in file managers and appimaged.
