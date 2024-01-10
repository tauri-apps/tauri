---
'tauri': 'patch:bug'
---

Preserve the order of JS object/map keys in IPC calls. This also fixes issues with the JS `http` module when calling to servers that required a specific order of `FormBody` contents.
