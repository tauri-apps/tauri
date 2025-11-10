---
"tauri": patch:bug
---

Fixed 500 error when accessing local video files in Android external storage directory via `convertFileSrc`. Added better error handling and logging for Android external storage access to help diagnose permission and accessibility issues.