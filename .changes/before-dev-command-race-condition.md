---
"tauri.js": patch
---

Fixes a race condition on the `beforeDevCommand` usage (starting Tauri before the devServer is ready).
