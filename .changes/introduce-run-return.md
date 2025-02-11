---
tauri: 'minor:feat'
tauri-runtime: 'minor:feat'
tauri-runtime-wry: 'minor:feat'
---

Add `App::run_return` function. Contrary to `App::run`, this will **not** exit the thread but instead return the requested exit-code. This allows the host app to perform further cleanup after Tauri has exited.
