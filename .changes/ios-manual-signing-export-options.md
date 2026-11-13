---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Read manual iOS signing certificates and provisioning profiles from the Xcode project when generating export options, without overriding the values configured in `ExportOptions.plist`. Provisioning profiles from `ExportOptions.plist` (such as the ones for app extensions) are now kept when merging.
