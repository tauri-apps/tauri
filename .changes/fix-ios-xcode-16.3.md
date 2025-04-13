---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fixes iOS dev not working on Xcode 16.3 simulators. To apply the fix, either regenerate the Xcode project with `rm -r src-tauri/gen/apple && tauri ios init` or remove the `arm64-sim` architecture from the Xcode project.
