---
"tauri": patch
---

Adjust the updater to fallback to `$HOME/.cache` or the current working directory as temp directory if the system default is in a different mount point.
