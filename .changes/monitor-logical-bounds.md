---
"@tauri-apps/api": "patch:enhance"
---

Document that `Monitor.size`, `Monitor.position` and `Monitor.workArea` are in physical pixels, with examples showing how to convert them to the logical pixels expected by window creation options via `toLogical(monitor.scaleFactor)`.
