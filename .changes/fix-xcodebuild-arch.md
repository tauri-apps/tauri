---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Do not include the target arch when building and archiving the iOS application,
which makes Xcode project modifications more flexible.
