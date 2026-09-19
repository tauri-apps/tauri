---
"tauri": minor:feat
"tauri-codegen": minor:feat
"tauri-utils": minor:feat
---

Added `Image::from_app_icon_resource` and `Image::from_icon_resource` on Windows for loading images from icon resources embedded in the executable (identified by an `IconResource` id or name), and the default `default_window_icon` from `tauri::generate_context` macro is now loaded using `from_app_icon_resource`. The resource id `tauri-build` embeds the application icon with is exposed as `tauri_utils::platform::WINDOWS_APP_ICON_RESOURCE_ID`.
