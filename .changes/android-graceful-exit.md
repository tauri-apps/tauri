---
'tauri': 'patch:bug'
---

On Android, `AppHandle::exit` (and the process plugin `exit` command) now finishes the activity instead of exiting the process directly, so the app closes with the system transition instead of flashing a blank screen.
