---
"tauri-build": "minor:feat"
---

Added `tauri_build::WindowsAttributes::static_vc_runtime` to control MSVC static runtime linking from build scripts. The `STATIC_VCRUNTIME` environment variable is now deprecated and emits a migration warning when used.
