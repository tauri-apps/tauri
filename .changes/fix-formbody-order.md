---
'tauri': 'patch:bug'
---

The IPC will now preserve the order of JS map keys. This fixes issues with servers that required a specific order of FormBody contents.
