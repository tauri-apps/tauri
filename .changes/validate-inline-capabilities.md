---
"tauri-build": patch:bug
---

Validate the capabilities inlined in the `app > security > capabilities` configuration value, which previously skipped the build script validation and only failed later with a generic `failed to resolve ACL` panic when a permission identifier was unknown.
