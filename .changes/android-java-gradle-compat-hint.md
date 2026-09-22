---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Warn during Android commands (`init`/`dev`/`build`) when the active Java version is too new for the Gradle version the project uses (e.g. Java 27 against the Gradle 9.6.1 the template ships, or Java 25 against a project still on Gradle 8.14), instead of letting the build fail later with a cryptic error. The warning points to the Gradle/Java compatibility matrix and suggests a supported JDK.
