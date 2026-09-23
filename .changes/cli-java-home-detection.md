---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix Java version detection for the Android Java/Gradle compatibility warning when `JAVA_HOME` is not set: the `java` binary on `PATH` is now resolved through symlinks (e.g. Linux alternatives and Homebrew), and on macOS the `/usr/bin/java` stub falls back to `/usr/libexec/java_home`.
