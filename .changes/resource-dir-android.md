---
"tauri-utils": patch:bug
---

Implemented `resource_dir` on Android, which returns a URI that needs to be resolved using [AssetManager::open](https://developer.android.com/reference/android/content/res/AssetManager#open(java.lang.String,%20int)). This will be handled by the file system plugin.
