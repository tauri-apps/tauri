---
"tauri": patch
---

`Notification.requestPermission()` now returns `"denied"` when not allowlisted.
`IsNotificationPermissionGranted` returns `false` when not allowlisted.
