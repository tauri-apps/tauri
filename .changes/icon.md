---
"tauri.js": patch
---

Update the tauri template to properly set the app icon id on Windows so the webview can load the executable icon.
To use it on old projects, update your `src-tauri/src/build.rs` file, replacing `res.set_icon("icons/icon.ico");` with `res.set_icon_with_id("icons/icon.ico", "32512");`.
