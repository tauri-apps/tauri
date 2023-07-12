---
'@tauri-apps/api': 'patch:fix'
---

Fix `Body.form` static not reading and sending entries of type `Blob` (including subclasses such as `File`)
