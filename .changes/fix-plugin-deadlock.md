---
"tauri": patch
---

Fixes deadlock when a plugin window ready event needs to block the thread waiting on the event loop.
