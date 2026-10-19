---
"tauri-bundler": patch:changes
"tauri-cli": patch:changes
"@tauri-apps/cli": patch:changes
---

Split the NSIS `installer.nsi` template into `.nsh` includes and replaced the register usage with named variables.

The built-in language strings now interpolate `$ExistingVersionKind`, `$Webview2DownloadResult` and `$Webview2InstallExitCode` instead of `$R4`, `$0` and `$1`. Custom language files using the registers keep working, but should be updated to the named variables.
