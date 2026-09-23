---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `PRODUCT_BUNDLE_IDENTIFIER` in the iOS Xcode project being set to the raw `identifier` (with underscores) instead of the sanitized iOS bundle identifier used by the Xcode project template and the export options provisioning profiles.
