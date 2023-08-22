---
"tauri": 'patch:bug'
---

No longer unpacking and flattening the `payload` over the IPC so that commands with an argument called `options` don't break.
