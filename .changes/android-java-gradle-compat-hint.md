---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Warn during Android commands (`init`/`dev`/`build`) when the active Java version is too new for the Gradle version Tauri ships (e.g. Java 25+ against Gradle 8.14), instead of letting the build fail later with a cryptic error. The warning points to the Gradle/Java compatibility matrix and suggests a supported JDK.
