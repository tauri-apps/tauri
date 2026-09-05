---
"tauri-bundler": minor:enhance
---

Switch to use restart manager to close running app, this makes it so that we send a `WM_ENDSESSION` signal to the app for it to gracefully shutdown
