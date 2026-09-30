---
"tauri": "minor:feat"
"@tauri-apps/api": "minor:feat"
---

Add `TrayIconBuilder::autosave_name` (the `autosaveName` option of the JavaScript `TrayIcon.new`), which sets `NSStatusItem.autosaveName` on macOS. macOS already restores where the user <kbd>Cmd</kbd>+dragged a tray icon on the next launch, keyed by the order in which the app created its tray icons (`Item-0`, `Item-1`, and so on); an autosave name replaces that generated key with a stable one, so an app that creates several tray icons, or that creates one conditionally, doesn't have icons come back holding each other's positions.
