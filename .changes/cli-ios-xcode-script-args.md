---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Quote the Tauri CLI path and the Xcode variables in the iOS "Build Rust Code" build phase, fixing builds when Xcode or the CLI is installed in a path containing spaces (requires regenerating the Xcode project with `tauri ios init`). Also fixes `CXXFLAGS` not being set for iOS targets in `tauri ios xcode-script`.
