---
"tauri": "patch:bug"
---

Re-register the Android `ActivityResultLauncher`s (permission requests, `startActivityForResult`, `startIntentSenderForResult`) whenever the activity is recreated, instead of keeping the ones bound to the previous activity. This fixes an `IllegalStateException: Attempting to launch an unregistered ActivityResultLauncher` crash that happened after the activity was recreated (e.g. on rotation or theme changes).
