---
"tauri": patch
---

Fixes a crash when a request is made to `https://tauri.$URL` on Windows where `$URL` is not `localhost/**` e.g. `https://tauri.studio`.
