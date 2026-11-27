---
'tauri-cli': 'minor:feat'
'@tauri-apps/cli': 'minor:feat'
---

Android release builds are now signed with the upload key referenced by `gen/android/keystore.properties` when the file exists. New projects include the signing configuration in `build.gradle.kts`, and it is added to existing projects by `tauri signer android generate-keystore` and `tauri signer android configure-keystore`.

`tauri android build` can also read the upload key from the `ANDROID_KEYSTORE` (base64 representation of the keystore) or `ANDROID_KEYSTORE_PATH`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` and `ANDROID_KEY_PASSWORD` environment variables, which removes the need to write `keystore.properties` in CI/CD platforms.
