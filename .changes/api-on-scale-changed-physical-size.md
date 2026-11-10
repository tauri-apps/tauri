---
"@tauri-apps/api": patch:bug
---

Fix `Window.onScaleChanged` passing `size` as a plain object instead of a `PhysicalSize`, so methods such as `toLogical` were missing on it. It is now wrapped the same way as the `onResized` payload.
