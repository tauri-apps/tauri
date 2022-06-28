---
"tauri-utils": patch
---

Removed `deny_unknown_fields` from the configuration objects. This causes issues when the Tauri CLI is updated but the core crates aren't, and it isn't harmful since the CLI validates the config schema.
