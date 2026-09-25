---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

The bundled `vswhere.exe` is now written to a new, uniquely named temporary file on each use and removed afterwards, instead of running any existing `%TEMP%\vswhere.exe`.
