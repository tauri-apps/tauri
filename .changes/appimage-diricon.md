---
tauri-bundler: patch:bug
tauri-cli: patch:bug
"@tauri-apps/cli": patch:bug
---

Fixed an issue in the AppImage bundler that caused the `/.desktop` and `.DirIcon` files to be absolute symlinks instead of relative symlinks which caused problems with AppImage installers like `AppManager`.
