---
"tauri": "minor:feat"
"tauri-utils": "minor:feat"
---

Add the `app > trayIcon > autosaveName` configuration option, which sets `NSStatusItem.autosaveName` for the configured tray icon on macOS, so that an app which creates more tray icons at runtime can keep the configured one's saved menu bar position tied to a stable key instead of to its creation order.
