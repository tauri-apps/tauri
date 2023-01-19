---
'tauri-bundler': 'minor'
---

Added support for pre-release identifiers and build numbers for the `.msi` bundle target. Only one of each can be used and it must be numeric only. The version must still be semver compatible according to https://semver.org/.
