---
'tauri-runtime-cef': 'patch:bug'
---

The Chrome command filter no longer governs browsers the webview does not own. CEF routes a DevTools window opened from F12 or the context menu's Inspect through the opener's client, so the filter was swallowing that window's own commands: because `zoom_hotkeys_enabled` defaults to false, Ctrl+Plus, Ctrl+Minus and Ctrl+0 did nothing in DevTools for essentially every app, and print, save page and new tab were dead there too. The handler now checks the browser identity the same way the display handler and the frame observer do, and leaves every other browser's commands to CEF. In a CEF-owned popup, which is a real Chrome window with an app menu, the commands the runtime does swallow are now reported as disabled instead of drawing as ordinary entries that do nothing when clicked.
