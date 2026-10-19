---
"tauri-bundler": "patch:bug"
---

Quote the executable path in the file association `shell\open\command` and `DefaultIcon` registry values written by the NSIS installer, so opening an associated file no longer depends on how Windows splits an unquoted path that contains spaces.
