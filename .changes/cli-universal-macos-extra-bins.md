---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri build --target universal-apple-darwin` failing at the bundling step when the app has more than one binary. Every binary is now merged with `lipo`, not only the main one.
