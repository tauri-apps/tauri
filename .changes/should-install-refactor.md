---
"tauri": patch
---

**Breaking change**: `UpdateBuilder::should_update` now takes the current version as a `semver::Version` and a `RemoteRelease` struct, allowing you to check other release fields.
