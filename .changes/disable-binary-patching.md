---
"tauri-bundler": "minor:feat"
"tauri-cli": "minor:feat"
---

Add a `--no-binary-patching` flag to `tauri build` and `tauri bundle`. When set, the bundler skips patching the main executable with bundle type information (and the subsequent re-signing), leaving an already-signed binary untouched. Patching is only required when shipping multiple bundle types per platform that should each update with their own installer format.
