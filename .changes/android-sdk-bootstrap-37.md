---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix the Android SDK auto installer setting up `platforms;android-36` while the generated app project compiles against `compileSdk = 37`, which made `tauri android build` fail right after the CLI finished installing the SDK.
