---
"api": patch
"tauri": patch
---

Fix missing asset protocol path.Now the protocol is `https://asset.localhost/path/to/file` on Windows. Lunix and macOS
is still `asset://path/to/file`.
