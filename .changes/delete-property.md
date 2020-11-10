---
"tauri.js": patch
---

Fixes `Reflect.deleteProperty` on promisified API calls failing with `Unable to delete property` by making it configurable.
