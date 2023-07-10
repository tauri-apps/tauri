---
'@tauri-apps/api': 'minor:feat'
---

Add `Body.formAsync` static method to the `http` module to create a `Body` from a `FormData` that respects entries of type `Blob` (including subclasses such as `File`)
