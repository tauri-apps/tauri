---
"@tauri-apps/api": "minor:feat"
---

Add `logicalPosition`, `logicalSize` and `logicalWorkArea` to the `Monitor` object returned by `currentMonitor()`, `primaryMonitor()`, `monitorFromPoint()` and `availableMonitors()`. These values are derived from the existing physical bounds and `scaleFactor`, and match the logical units expected by window creation options, removing the need for manual conversion (especially on macOS mixed-DPI setups).
