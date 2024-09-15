---
"tauri": patch:breaking
"tauri-runtime-wry": patch:breaking
---

Change the pointer type of `PlatformWebview`'s `inner`, `controller`, `ns_window` and `view_controller` to `c_void`, to avoid publically depending on `objc`.
