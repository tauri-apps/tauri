---
'tauri-runtime': 'minor:breaking'
'tauri-runtime-wry': 'minor:breaking'
---

`Dispatch::create_window`, `Runtime::create_window` and `RuntimeHandle::create_window` has been changed to accept a 3rd parameter which is a closure that takes `RawWindow` and to be executed right after the window is created and before the webview is added to the window.
