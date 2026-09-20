---
"tauri-cli": "patch:enhance"
"@tauri-apps/cli": "patch:enhance"
---

Align the `tauri plugin new` mobile templates with the app template: the Android library now uses `compileSdk = 37` and the iOS templates target iOS 15.0 (`IPHONEOS_DEPLOYMENT_TARGET` for the Xcode template, `.iOS(.v15)` for the Swift Package template, which now requires swift-tools-version 5.5).
