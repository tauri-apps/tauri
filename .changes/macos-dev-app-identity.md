---
'tauri': 'patch:bug'
'tauri-utils': 'patch:bug'
'tauri-codegen': 'patch:bug'
---

On macOS, run the development binary from a minimal app bundle next to it (`target/<profile>/.tauri-dev/<binary>.app`) so it gets its own Dock icon and app switcher entry, instead of being registered under the app it was launched from (e.g. Terminal or VS Code).
