---
'tauri-bundler': 'minor:feat'
'tauri-cli': 'minor:feat'
'@tauri-apps/cli': 'minor:feat'
---

Added the `APPLE_API_KEY_CONTENT` environment variable to provide the App Store Connect API private key contents (or its base64 representation) for notarization and iOS automatic signing, as an alternative to writing the key to a file referenced by `APPLE_API_KEY_PATH`.
