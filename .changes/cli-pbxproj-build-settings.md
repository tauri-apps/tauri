---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix iOS Xcode project synchronization corrupting `project.pbxproj` when several build settings were added to the same configuration, when an added setting was changed again, or when a multi-line setting was overwritten. Product names, signing identities and other values written to the project are now properly escaped.
