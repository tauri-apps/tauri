---
"tauri-runtime-cef": patch:bug
---

Fix `App::set_theme` and `Window::set_theme` on macOS updating Chromium's theme without updating the application appearance in AppKit. AppKit UI now follows the selected light or dark theme.
