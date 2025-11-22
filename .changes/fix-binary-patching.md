---
"tauri": patch:perf
"tauri-cli": patch:perf
"tauri-bundler": patch:perf
"@tauri-apps/cli": patch:perf
---

Change the way bundle type information is added to binary files. Intead of looking up value of a variable we simply look for default value.
