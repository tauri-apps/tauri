---
"tauri-cli": patch:feat
"@tauri-apps/cli": patch:feat
---

Setup Android signing by reading the `src-tauri/gen/android/keystore.properties` file which should have the `keyAlias=<key-alias>`, `password=<keystore-password>`, `storeFile=<path/to/keystore.jks>` key value pairs separated by new lines.
