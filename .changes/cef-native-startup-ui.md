---
'tauri-runtime-cef': 'patch:enhance'
---

Expose macOS CEF application preparation so native recovery dialogs can open before the runtime or browser profile initializes, without creating an incompatible AppKit application singleton.

Retain browser windows until CEF acknowledges application shutdown, so queued native child-view destruction can complete and the process can exit. Close attached DevTools as part of that shutdown.
