---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri init` no longer panics when the app name, window title, frontend dist, dev URL or before dev/build commands contain quotes or backslashes (e.g. `-D ..\dist` or `vite --host "0.0.0.0"`); the values are now JSON-escaped in the generated `tauri.conf.json`.
