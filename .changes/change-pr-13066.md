---
"@tauri-apps/api": patch
"tauri-utils": patch
"tauri-macos-sign": patch
"tauri-bundler": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
"tauri-codegen": patch
"tauri-macros": patch
"tauri-plugin": patch
"tauri-build": patch
"tauri": patch
"@tauri-apps/cli": patch
"tauri-cli": patch
"tauri-driver": patch
---
fix 
---
"@tauri-apps/api": patch:enhance
---
Add a generic to `emit` and `emitTo` functions for the `payload` instead of the previously used type (`unknown`).
