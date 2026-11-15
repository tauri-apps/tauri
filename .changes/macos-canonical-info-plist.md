---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Canonicalize macOS app and CEF helper Info.plists with Apple's XML serializer before signing, fixing Chromium dynamic signature validation failing with `errSecCSInfoPlistFailed` (`-67030`) despite a valid on-disk signature.
