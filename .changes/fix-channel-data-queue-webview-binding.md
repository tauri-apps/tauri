---
"tauri": patch:sec
---

Bind channel data IPC queue entries to the webview they were created for and scope their ids per webview.
Queued channel payloads and large invoke responses can no longer be read by other webviews, and entries are purged when the owning webview is closed.
Fixes GHSA-w28w-mhc8-qvjv.
