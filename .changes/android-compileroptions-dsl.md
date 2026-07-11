---
"tauri": patch:enhance
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Migrate the Android Gradle scripts from the deprecated `kotlinOptions` DSL to `compilerOptions`, which is accepted by both Kotlin Gradle Plugin 1.9.x and 2.x. This lets projects move to Kotlin 2.x without hitting the hard error that 2.3+ raises on the old DSL.
