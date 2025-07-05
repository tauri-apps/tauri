---
tauri-bundler: patch:bug
---

Fix incorrect expected file path for `nsis_tauri_utils.dll` resulting in tauri-cli re-downloading the file on every build.
