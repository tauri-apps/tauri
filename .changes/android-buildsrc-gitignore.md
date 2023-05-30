---
"tauri-cli": patch
"@tauri-apps/cli": patch
---

Do not gitignore the Android project's `buildSrc` folder by default since we removed absolute paths from it.
