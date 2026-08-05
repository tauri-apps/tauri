---
"tauri-cli": "minor:feat"
"@tauri-apps/cli": "minor:feat"
"tauri-utils": "minor:feat"
---

Added `build.windows.staticVCRuntime` to control MSVC static runtime linking. The `STATIC_VCRUNTIME` environment variable is now deprecated and emits a migration warning when used.
