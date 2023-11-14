---
"@tauri-apps/cli": patch:bug
"tauri-cli": patch:bug
"tauri-bundler": patch:bug
---

Fixes errors on command output, occuring when the output stream contains an invalid UTF-8 character, or ends with a multi-bytes UTF-8 character.
