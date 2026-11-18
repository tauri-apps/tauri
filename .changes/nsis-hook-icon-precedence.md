---
"tauri-bundler": patch:bug
---

`MUI_ICON` and `MUI_UNICON` defined by an NSIS installer hook now take precedence over `bundle.windows.nsis.installerIcon` and `uninstallerIcon` instead of failing the build with an "already defined" error.
