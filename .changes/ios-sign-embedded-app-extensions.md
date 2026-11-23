---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Preserve the entitlements of embedded app extensions (and other nested bundles such as watch apps and app clips) when building an iOS app with App Store Connect API key authentication. Previously only the main app binary was signed before export, so extensions silently lost entitlements such as App Groups.
