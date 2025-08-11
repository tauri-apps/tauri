---
"tauri-runtime-wry": "patch:bug"
---

Use WM_NCDESTROY instead of WM_DESTROY to free window userdata, fixing a double-free occurring in the Windows resizing handler for undecorated windows which caused STATUS_HEAP_CORRUPTION
