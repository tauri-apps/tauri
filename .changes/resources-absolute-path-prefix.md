---
"tauri-utils": patch:bug
---

Fixed absolute paths in `bundle > resources` losing their prefix on Windows. `C:\assets\data.json` was read back as `\assets\data.json`, which resolves against the drive of the current directory, so the resource was silently copied from the wrong drive or failed with `ResourcePathNotFound`.
