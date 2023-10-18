---
"tauri": 'patch:bug'
---

No longer unpacking and flattening the `payload` over the IPC so that commands with arguments called `cmd`, `callback`, `error`, `options` or `payload` aren't breaking the IPC.
