---
"tauri-bundler": minor:feat
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
---

Added support for `minimumWebview2Version` option support for the MSI (Wix) installer, the old `bundle > windows > nsis > minimumWebview2Version` is now deprecated in favor of `bundle > windows > minimumWebview2Version`

Notes:

- For anyone relying on the `WVRTINSTALLED` `Property` tag in `main.wxs`, it is now renamed to `INSTALLED_WEBVIEW2_VERSION`
- For `tauri-bundler` lib users, the `WindowsSettings` now has a new field `minimum_webview2_version` which can be a breaking change
