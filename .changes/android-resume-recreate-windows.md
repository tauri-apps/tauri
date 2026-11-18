---
"tauri": "patch:bug"
---

On Android, re-create the configured windows when the app is resumed with no webviews alive. This fixes a blank screen when the app is relaunched after task removal while a foreground service keeps the process alive.
