---
"tauri-runtime-cef": patch:deps
---

Bump the `windows` crate to 0.62 to match `tauri-runtime`, fixing the Windows build (`HWND` type mismatch in the `WindowBuilder::owner`/`parent` implementations).
