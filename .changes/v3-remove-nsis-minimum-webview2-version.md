---
"tauri-utils": major:breaking
"tauri-bundler": major:breaking
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

Removed the deprecated `bundle > windows > nsis > minimumWebview2Version` config option (`NsisConfig::minimum_webview2_version`) and `NsisSettings::minimum_webview2_version`. Use `bundle > windows > minimumWebview2Version` instead, which now applies to the NSIS installer even when no `nsis` config object is present.
