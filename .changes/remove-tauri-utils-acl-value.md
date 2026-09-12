---
tauri-utils: minor:breaking
---

Removed `acl::Value` in favor of `serde_json::Value`. `Scopes` and `ResolvedScope` now hosts `serde_json::Value` instead.
