---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Escape special characters in `productName` when generating Android `strings.xml`, fixing build failures when the name contains single quotes or other XML-unsafe characters.
