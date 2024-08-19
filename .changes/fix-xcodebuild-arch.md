---
'tauri-cli': 'patch:fix'
'@tauri-apps/cli': 'patch:fix'
---

Do not include the target arch when building and archiving the iOS application,
which makes Xcode project modifications more flexible.
