---
"tauri-utils": patch:bug
---

Keep the drive letter or UNC share of absolute `bundle > resources` and `bundle > externalBin` paths on Windows. Before, a resource on another drive than the current directory, for example `D:/assets/*` in a project on `C:`, failed with a "resource path doesn't exist" error.
