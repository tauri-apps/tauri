---
"tauri-codegen": patch:bug
"tauri-runtime-wry": patch:bug
---

On Windows, set `ICON_BIG` from the exe-embedded ICO resource via `IconExtWindows::from_resource` for crisp taskbar and alt-tab icons. Cap the codegen ICO entry at 32x32 for `ICON_SMALL` to reduce binary size.
