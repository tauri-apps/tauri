---
"tauri-runtime-cef": patch:bug
---

Fixed app windows on Linux having no X11 window class (`WM_CLASS`). It is now the app's `identifier`.
