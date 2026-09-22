---
tauri-utils: minor:breaking
---

The `WindowEffect` enum is now `#[non_exhaustive]` so new effects can be added without a breaking change. Exhaustive `match` statements on it must add a wildcard arm.
