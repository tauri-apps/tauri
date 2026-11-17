---
"tauri-runtime-cef": minor:feat
"tauri-bundler": minor:feat
---

Added `Cef::frame_hosts_without_ipc`: frames whose document URL is on one of the given hosts, and every frame nested in them, get no `window.ipc`, so an application can embed untrusted documents (served from a custom scheme, for instance) without exposing the IPC channel to them. The macOS `tauri-cef-helper` the bundler compiles reads the same list.
