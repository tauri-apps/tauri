---
tauri: minor:deps
tauri-cli: minor:deps
"tauri-apps/cli": minor:deps
---

On Android, fix missing `consumer-rules.pro` file in the template.

**IMPORTANT**: For plugin authors, update your `build.gradle.kts` file to remove the

```kotlin
    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
```

section and rename your `proguard-rules.pro` to `consumer-rules.pro` to match the `consumerProguardFiles("consumer-rules.pro")` in the template.
