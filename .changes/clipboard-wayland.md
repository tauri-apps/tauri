---
'tauri-runtime-wry': 'patch'
---

Fix panic during intialization on wayland when using `clipboard` feature, instead propagate the error during API usage.
