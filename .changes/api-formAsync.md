---
'@tauri-apps/api': 'patch:bug'
---

Fix `Body.form` static not reading and sending entries of type `Blob` (including subclasses such as `File`)
