---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Normalize `gen/android/gradlew` CRLF line endings to LF on all host platforms, not only Unix. A `gradlew` checked out with CRLF broke `sh ./gradlew` on Windows hosts using Git Bash. The rewrite only runs when a CRLF is actually present. A failure to rewrite aborts on Unix, where the script is executed directly; on Windows the CLI invokes `gradlew.bat`, so failures there only warn.
