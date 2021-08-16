---
"tauri": minor
---

Move items which `tauri::api` re-exports from `tauri-utils` to individual module `utils`. Because these items has their
own Error/Result types which are not related to api module at all.

