---
"tauri-utils": patch:bug
---

Fix Tauri iOS build with binary XCFramework dependencies, allows extracting binaryTargets that are zipped and also not including XCFrameworks when linking.
