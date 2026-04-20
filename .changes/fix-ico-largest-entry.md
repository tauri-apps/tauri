---
"tauri-codegen": patch:bug
"tauri-runtime-wry": patch:bug
---

On Windows, load the window icon from the exe-embedded resource so both the titlebar (`ICON_SMALL`) and taskbar (`ICON_BIG`) use the multi-size ICO instead of a single RGBA buffer. Also select the largest ICO entry in codegen as a fallback.
