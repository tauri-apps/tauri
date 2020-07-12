---
"tauri.js": minor
---

Move types exported in the `tauri` js api into the modules that use them. For
example, `Event` is now available from `tauri/api/event` instead of
`tauri/api/types/event`.
