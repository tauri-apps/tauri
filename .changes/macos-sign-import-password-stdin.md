---
"tauri-macos-sign": patch:enhance
---

Pass the signing certificate password to `security import` through stdin instead of the command line, so it no longer shows up in the process list or in debug logs.
