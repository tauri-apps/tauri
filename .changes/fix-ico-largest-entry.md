---
"tauri-codegen": patch:bug
---

Select the largest entry of an ICO file instead of the first one when embedding it with `include_image!` or as the tray icon, so the OS downscales a high-resolution source instead of upscaling the 16x16 entry.
