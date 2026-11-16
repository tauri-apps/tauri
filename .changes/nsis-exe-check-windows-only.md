---
"tauri-bundler": "patch:bug"
---

Fix NSIS bundling on macOS and Linux deleting the cached NSIS tools and downloading `nsis_tauri_utils.dll` again on every build. The `NSIS.exe` hash check now only runs on Windows, where `NSIS.exe` is part of the downloaded toolset.
