---
tauri-runtime-wry: patch:bug
---

Fix getting the DPI internally leaks `HDC` handles on Windows. This also improves the resizing speed on undecorated windows
