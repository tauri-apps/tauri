---
'tauri-cli': 'patch:feat'
'@tauri-apps/cli': 'patch:feat'
---

Allow Xcode to manage iOS code sign and provisioning profiles by default.
On CI, the `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_KEY_PATH` environment variables must be provided for authentication.
