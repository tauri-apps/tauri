---
"tauri-bundler": "minor:feat"
"tauri-cli": "minor:feat"
"@tauri-apps/cli": "minor:feat"
"tauri-utils": "minor:feat"
---

Added `bundle.windows.bundleVCRuntime` to copy the Visual C++ runtime DLLs into Windows MSI and NSIS installers. The bundler locates the runtime through `VCTOOLS_REDIST_DIR` or the bundled `vswhere.exe`.
