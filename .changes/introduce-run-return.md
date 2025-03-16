---
tauri: 'minor:feat'
tauri-runtime: 'minor:feat'
tauri-runtime-wry: 'minor:feat'
---

Add `App::run_return` function. Contrary to `App::run`, this will **not** exit the process but instead return the requested exit-code. This allows the host app to perform further cleanup after Tauri has exited. `App::run_return` is not available on iOS and fallbacks to the regular `App::run` functionality.

The `App::run_iteration` function is deprecated as part of this because calling it in a loop - as suggested by the name - will cause a busy-loop.
