---
"tauri-macos-sign": patch:deps
"tauri-runtime-wry": patch:deps
---

Replace `once_cell` with the `std::sync` equivalents, removing it from the direct dependencies.
