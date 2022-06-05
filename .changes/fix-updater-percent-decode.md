---
"tauri": patch
---

Fixes an issue with the updater when replacing the `{{target}}` and `{{current_version}}` variables due to percent-encoding on the `Url` that is parsed from the configuration.
