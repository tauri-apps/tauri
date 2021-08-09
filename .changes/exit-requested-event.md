---
"tauri": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
---

Add `ExitRequested` event that allows preventing the app from exiting when all windows are closed, and an `AppHandle.exit()` function to exit the app manually.
