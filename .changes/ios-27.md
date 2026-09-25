---
tauri-cli: minor:deps
"@tauri-apps/cli": minor:deps
---

Update the generated iOS Xcode project to build with Xcode 27. The scenes lifecycle is **not** enabled by default: to opt in, add the `UIApplicationSceneManifest` key to your `src-tauri/Info.ios.plist`.
