---
"tauri": minor:feat
"tauri-utils": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
---

Added `app > androidHostname` config to serve the Android app from `https://<androidHostname>` instead of `http(s)://tauri.localhost`, so autofill services can link webview credentials to the app through Digital Asset Links. Setting it forces `useHttpsScheme` to `true`.
