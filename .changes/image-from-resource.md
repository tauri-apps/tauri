---
"tauri": minor:feat
"tauri-build": minor:feat
"tauri-codegen": minor:feat
"tauri-macros": minor:feat
---

Added `Image::from_app_icon` and `Image::from_resource` on Windows for loading images from resources embedded in the executable, and the default `default_window_icon` from `tauri::generate_context` macro is now loaded using `from_app_icon`
