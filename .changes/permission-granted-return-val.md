---
"api": patch
---

The notification's `isPermissionGranted` function now returns `boolean` instead of `boolean | null`. The response is never `null` because we won't check the permission for now, always returning `true` instead.
