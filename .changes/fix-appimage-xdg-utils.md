---
tauri-bundler: patch:bug
tauri-cli: patch:bug
"@tauri-apps/cli": patch:bug
---

On Linux, do not bundle xdg-open and xdg-utils in the AppImage anymore. This rarely worked and usually requires host system support anyway.
