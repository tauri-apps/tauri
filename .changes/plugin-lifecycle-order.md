---
"tauri": "major:breaking"
---

A plugin is only added to the app once its `initialize` (or `setup`) returns, so its own hooks do not run during it. A removed plugin is dropped once its hooks that are already running return.
