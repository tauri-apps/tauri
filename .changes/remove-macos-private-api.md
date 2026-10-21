---
"tauri": major:breaking
"tauri-utils": major:breaking
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
"tauri-runtime-cef": major:breaking
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

Removed the `macos-private-api` Cargo feature and the `app > macOSPrivateApi` configuration option. Window transparency and the `fullScreenEnabled` preference no longer rely on macOS private APIs, so they are always available.
