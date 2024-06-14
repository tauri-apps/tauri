---
"tauri-bundler": "patch:breaking"
---

Changed NSIS start menu shortcut to be placed directly inside `%AppData%\Microsoft\Windows\Start Menu\Programs` without an additional folder. You can get the old behavior by setting `bundle > nsis > startMenuFolder` to the same value as your `productName`
