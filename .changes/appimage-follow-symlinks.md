---
'tauri-bundler': patch:bug
---

- Updated the AppImage bundler to follow symlinks for `/usr/lib*`.
- Fixes AppImage bundling for Void Linux, which was failing to bundle webkit2gtk because the `/usr/lib64` is a symlink to `/usr/lib`.
