---
'tauri': 'patch'
---

Revert [#6680](https://github.com/tauri-apps/tauri/pull/6680) which added a default sound for notifications on Windows. This introduced inconsistency with other platforms that has silent notifications by default. In the upcoming releases, we will add support for modifying the notification sound across all platforms.
