---
"tauri-build": patch
---

Only statically link the VC runtime when the `STATIC_VCRUNTIME` environment variable is set to `true` (automatically done by the Tauri CLI).
