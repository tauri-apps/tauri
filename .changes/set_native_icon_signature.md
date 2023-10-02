---
"tauri": 'patch:bug'
---

Changed `IconMenuItem::set_native_icon` signature to take `&self` instead of `&mut self` to fix compilation error on macos.
