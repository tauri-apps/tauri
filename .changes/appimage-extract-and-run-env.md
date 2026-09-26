---
"tauri-utils": "patch:bug"
---

No longer warn about a possible security issue when an AppImage runs through `--appimage-extract-and-run`, which extracts to `$TMPDIR/appimage_extracted_*` instead of mounting under `$TMPDIR/.mount_*`.
