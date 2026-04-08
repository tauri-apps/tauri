---
"tauri-bundler": minor:feat
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
"tauri-utils": minor:feat
---

Added uninstaller icon and uninstaller header image support for NSIS installer.

Notes:

- For `tauri-bundler` lib users, the `NsisSettings` now has 2 new fields `uninstaller_icon` and `uninstaller_header_image` which can be a breaking change
- When bundling with NSIS, users can add `uninstallerIcon` and `uninstallerHeaderImage` under `bundle > windows > nsis` to configure them.
