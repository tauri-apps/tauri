---
tauri-build: 'patch:bug'
tauri-bundler: 'patch:bug'
---

The executable and NSIS installer on Windows will now use the `productName` config for the `FileDescription` property instead of `shortDescription`.
