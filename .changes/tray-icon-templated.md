---
"tauri": "minor:feat"
"@tauri-apps/api": "minor:feat"
---

Add `TrayIconBuilder::icon_templated` and `TrayIcon::set_icon_templated` (`TrayIcon.setIconTemplated` in JavaScript), which set the tray icon and draw it as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image in one call, so the template flag can never be left describing an icon that has since been replaced. Also add `TrayIcon::icon_is_template` to read that state back.

Deprecate `TrayIconBuilder::icon_as_template`, `TrayIcon::set_icon_as_template` (`TrayIcon.setIconAsTemplate` in JavaScript) and `TrayIcon::set_icon_with_as_template` (`TrayIcon.setIconWithAsTemplate` in JavaScript) in favour of them.
