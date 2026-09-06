---
'tauri-runtime-cef': 'patch:enhance'
---

Expose macOS CEF application preparation so native recovery dialogs can open before the runtime or browser profile initializes, without creating an incompatible AppKit application singleton.
