---
"tauri-macos-sign": patch:bug
---

Delete the temporary signing keychain when importing the certificate or resolving the signing identity fails, instead of leaving it behind in `~/Library/Keychains`.
