---
"tauri": patch
---

- Do not run the updater with UAC task if server don't tell us. (Allow toggling server-side)
- The updater expect a field named `with_elevated_task` with a `boolean` and will not run if the task is not installed first. (windows only)
