---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
---

Removed alpha channel from default icons in iOS template to comply with Apple's human interface guideline
(https://developer.apple.com/design/human-interface-guidelines/app-icons), because
transparent icons with alpha channel are not allowed, and will be rejected
upon upload to Apple appstore.
