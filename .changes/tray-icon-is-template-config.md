---
"tauri": "minor:feat"
"tauri-utils": "minor:feat"
"@tauri-apps/api": "minor:feat"
---

Add the `app > trayIcon > iconIsTemplate` configuration option and the matching `iconIsTemplate` option of the JavaScript `TrayIcon.new`, which draw the configured tray icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image on macOS.

Deprecate the previous `iconAsTemplate` names (`TrayIconConfig::icon_as_template` in Rust) in favour of them. Both names keep working, and either one set to `true` draws the icon as a template.
