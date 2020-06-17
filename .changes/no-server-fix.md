---
"tauri": minor
"tauri-api": minor
---

Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
Build no longer fails when compiling without environment variables, now the app will show an error.
