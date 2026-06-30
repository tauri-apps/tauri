---
"tauri-utils": "minor:feat"
"tauri-bundler": "minor:feat"
"tauri-cli": "minor:feat"
---

Add a `bundle > disableBinaryPatching` config option. When enabled, the bundler skips patching the main executable with bundle type information (and the subsequent re-signing), leaving an already-signed binary untouched. Patching is only required when shipping multiple bundle types per platform that should each update with their own installer format.
