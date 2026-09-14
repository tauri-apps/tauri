---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Normalize `gen/android/gradlew` CRLF line endings to LF on all host platforms, not only Unix. A `gradlew` checked out with CRLF broke `sh ./gradlew` on Windows hosts using Git Bash. The rewrite only runs when a CRLF is actually present, and read or write failures are logged instead of aborting the build, since the Windows CLI invokes `gradlew.bat` anyway.
